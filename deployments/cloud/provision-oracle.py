#!/usr/bin/env python3
"""
Oracle Cloud Always Free - Udaya 3-Node Deployment
Provisions 3 ARM VMs and deploys Udaya testnet nodes.

Prerequisites:
    pip install oci
    oci setup config   # configure tenancy, user, key, region

Usage:
    python provision-oracle.py --compartment-id <COMPARTMENT_OCID> --region <REGION> \
        --ssh-key-path ~/.ssh/id_rsa.pub --binary-path ./Udayad
"""

import argparse
import json
import os
import subprocess
import sys
import time
import uuid
from pathlib import Path

try:
    import oci
    from oci.core import ComputeClient, VirtualNetworkClient
    from oci.core.models import (
        AddImageDetails,
        AttachVnicDetails,
        BootVolumeAttachment,
        CreateImageDetails,
        CreateInstanceDetails,
        CreateVcnDetails,
        InstanceSourceViaImageDetails,
        LaunchInstanceDetails,
        LaunchOptions,
        VnicAttachment,
    )
except ImportError:
    print("ERROR: oci package not installed. Run: pip install oci")
    sys.exit(1)


def get_config():
    """Load OCI config from standard location."""
    config_path = os.path.expanduser("~/.oci/config")
    if not os.path.exists(config_path):
        print(f"ERROR: OCI config not found at {config_path}")
        print("Run: oci setup config")
        sys.exit(1)
    config = oci.config.from_file(config_path)
    oci.config.validate_config(config)
    return config


def get_compartment_id(config):
    """Get root compartment ID from tenancy."""
    identity_client = oci.identity.IdentityClient(config)
    tenancy_id = config["tenancy"]
    tenancy = identity_client.get_tenancy(tenancy_id).data
    return tenancy.id


def find_available_arm_image(config, compartment_id):
    """Find latest Oracle Linux ARM image."""
    compute_client = ComputeClient(config)
    images = compute_client.list_images(
        compartment_id=compartment_id,
        shape="VM.Standard.A1.Flex",
        operating_system="Oracle Linux",
        sort_by="TIMECREATED",
        sort_order="DESC",
        limit=5,
    ).data

    if not images:
        print("ERROR: No ARM images found. Check compartment and region.")
        sys.exit(1)

    for img in images:
        if img.lifecycle_state == "AVAILABLE":
            print(f"  Using image: {img.display_name} ({img.id})")
            return img.id

    print("ERROR: No available ARM images found")
    sys.exit(1)


def wait_for_instance(compute_client, instance_id, target_state="RUNNING", timeout=600):
    """Wait for instance to reach target state."""
    start = time.time()
    while time.time() - start < timeout:
        instance = compute_client.get_instance(instance_id).data
        if instance.lifecycle_state == target_state:
            return instance
        elif instance.lifecycle_state in ("TERMINATING", "TERMINATED"):
            raise RuntimeError(f"Instance {instance_id} entered state {instance.lifecycle_state}")
        time.sleep(15)
    raise TimeoutError(f"Instance {instance_id} did not reach {target_state} within {timeout}s")


def get_public_ip(vnic_client, vnic_id):
    """Get public IP for a VNIC."""
    vnic = vnic_client.get_vnic(vnic_id).data
    if vnic.public_ip:
        return vnic.public_ip
    # If no ephemeral public IP, check reserved IP
    if vnic.hostname_label:
        return vnic.hostname_label
    return "pending"


def provision_infrastructure(config, compartment_id, region, ssh_public_key):
    """Provision VCN, subnet, and 3 compute instances."""
    compute_client = ComputeClient(config, region=region)
    vnic_client = VirtualNetworkClient(config, region=region)

    # Find ARM image
    print("\n[1/5] Finding ARM image...")
    image_id = find_available_arm_image(config, compartment_id)

    # Create VCN
    print("[2/5] Creating VCN...")
    vcn_name = f"udaya-vcn-{uuid.uuid4().hex[:8]}"
    vcn = vnic_client.create_vcn(
        CreateVcnDetails(
            cidr_block="10.0.0.0/16",
            display_name=vcn_name,
            compartment_id=compartment_id,
            dns_label="udaya",
        )
    ).data
    print(f"  VCN: {vcn.id} ({vcn.cidr_block})")

    # Create Internet Gateway
    ig = vnic_client.create_internet_gateway(
        oci.core.models.CreateInternetGatewayDetails(
            display_name=f"udaya-igw-{uuid.uuid4().hex[:8]}",
            compartment_id=compartment_id,
            is_enabled=True,
            vcn_id=vcn.id,
        )
    ).data
    print(f"  IGW: {ig.id}")

    # Create Route Table
    route_table = vnic_client.create_route_table(
        oci.core.models.CreateRouteTableDetails(
            display_name=f"udaya-rt-{uuid.uuid4().hex[:8]}",
            compartment_id=compartment_id,
            vcn_id=vcn.id,
            route_rules=[
                oci.core.models.RouteRule(
                    destination="0.0.0.0/0",
                    destination_type="CIDR_BLOCK",
                    network_entity_id=ig.id,
                )
            ],
        )
    ).data

    # Create Subnet
    subnet = vnic_client.create_subnet(
        oci.core.models.CreateSubnetDetails(
            display_name=f"udaya-subnet-{uuid.uuid4().hex[:8]}",
            compartment_id=compartment_id,
            vcn_id=vcn.id,
            cidr_block="10.0.1.0/24",
            route_table_id=route_table.id,
            dns_label="subnet",
        )
    ).data
    print(f"  Subnet: {subnet.id} ({subnet.cidr_block})")

    # Create instances
    print("[3/5] Creating 3 ARM instances (Always Free)...")
    instances = []
    for i in range(1, 4):
        node_name = f"udaya-node{i}"
        print(f"  Creating ${node_name}...")
        instance = compute_client.launch_instance(
            LaunchInstanceDetails(
                compartment_id=compartment_id,
                display_name=node_name,
                shape="VM.Standard.A1.Flex",
                shape_config=oci.core.models.LaunchInstanceShapeConfigDetails(
                    ocpus=1,
                    memory_in_gbs=6,
                ),
                source_details=InstanceSourceViaImageDetails(
                    image_id=image_id,
                    boot_volume_size_in_gbs=50,
                ),
                create_vnic_details=oci.core.models.LaunchCreateVnicDetails(
                    assign_public_ip=True,
                    subnet_id=subnet.id,
                    hostname_label=f"udaya-node{i}",
                ),
                metadata={
                    "ssh_authorized_keys": ssh_public_key.strip(),
                },
                availability_domain=None,
            )
        ).data
        instances.append(instance)
        print(f"    ${node_name}: {instance.id} (state: {instance.lifecycle_state})")

    # Wait for instances to be running
    print("[4/5] Waiting for instances to be ready...")
    public_ips = []
    for i, instance in enumerate(instances, 1):
        print(f"  Waiting for udaya-node{i}...")
        running = wait_for_instance(compute_client, instance.id)
        # Get VNIC and public IP
        vnics = compute_client.list_vnic_attachments(
            compartment_id=compartment_id,
            instance_id=instance.id,
        ).data
        for vnic_att in vnics:
            if vnic_att.vnic_id:
                pub_ip = get_public_ip(vnic_client, vnic_att.vnic_id)
                public_ips.append(pub_ip)
                print(f"    udaya-node{i} public IP: {pub_ip}")
                break

    print("[5/5] Infrastructure provisioned!")
    return {
        "vcn_id": vcn.id,
        "subnet_id": subnet.id,
        "igw_id": ig.id,
        "instances": [
            {"id": inst.id, "name": inst.display_name, "ip": ip}
            for inst, ip in zip(instances, public_ips)
        ],
    }


def generate_env_file(infra, rpc_password, node_num):
    """Generate environment file for a node."""
    inst = infra["instances"][node_num - 1]
    return f"""# Udaya Node {node_num} Environment
export RPC_PASSWORD="{rpc_password}"
export NODE_EXTERNAL_IP="{inst['ip']}"
export NODE_ID="{inst['id']}"
export NODE_NAME="{inst['name']}"
"""


def main():
    parser = argparse.ArgumentParser(description="Provision Udaya 3-node testnet on Oracle Cloud Always Free")
    parser.add_argument("--compartment-id", required=False, help="OCI Compartment OCID")
    parser.add_argument("--region", required=False, default="us-ashburn-1", help="OCI Region")
    parser.add_argument("--ssh-key-path", required=False, default=os.path.expanduser("~/.ssh/id_rsa.pub"))
    parser.add_argument("--binary-path", required=False, default="./Udayad", help="Path to compiled Udayad binary")
    parser.add_argument("--rpc-password", required=False, default="udaya-testnet-rpc-change-me", help="RPC password")
    parser.add_argument("--output-dir", required=False, default="./deployments/cloud", help="Output directory for connection info")
    args = parser.parse_args()

    print("=" * 60)
    print("  Udaya Oracle Cloud Always Free Provisioning")
    print("=" * 60)

    # Load config
    print("\nLoading OCI configuration...")
    config = get_config()

    # Determine compartment
    if args.compartment_id:
        compartment_id = args.compartment_id
    else:
        compartment_id = get_compartment_id(config)
        print(f"  Using root compartment: {compartment_id}")

    # Read SSH key
    ssh_key_path = Path(args.ssh_key_path)
    if not ssh_key_path.exists():
        print(f"ERROR: SSH public key not found at {ssh_key_path}")
        print("Generate one with: ssh-keygen -t rsa -b 4096")
        sys.exit(1)
    ssh_public_key = ssh_key_path.read_text().strip()

    # Check binary exists
    binary_path = Path(args.binary_path)
    if not binary_path.exists():
        print(f"WARNING: Binary not found at {binary_path}")
        print("You will need to transfer the binary to each VM manually.")

    # Provision
    infra = provision_infrastructure(config, compartment_id, args.region, ssh_public_key)

    # Save connection info
    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    connection_info = {
        "region": args.region,
        "compartment_id": compartment_id,
        "vcn_id": infra["vcn_id"],
        "subnet_id": infra["subnet_id"],
        "nodes": [],
    }

    print("\n[FINAL] Connection Information:")
    print("-" * 60)
    for i, node in enumerate(infra["instances"], 1):
        print(f"  Node {i}: {node['name']}")
        print(f"    IP:       {node['ip']}")
        print(f"    ID:       {node['id']}")
        print(f"    SSH:      ssh -i ~/.ssh/id_rsa ubuntu@{node['ip']}")
        print(f"    P2P:      {node['ip']}:$((19797 + i))")
        print(f"    RPC:      {node['ip']}:$((18330 + i))  (localhost only)")
        print()

        # Save env file
        env_content = generate_env_file(infra, args.rpc_password, i)
        (output_dir / f"node{i}-env.sh").write_text(env_content)

        connection_info["nodes"].append({
            "name": node["name"],
            "id": node["id"],
            "ip": node["ip"],
            "ssh": f"ssh -i ~/.ssh/id_rsa ubuntu@{node['ip']}",
            "p2p_port": 19797 + i,
            "rpc_port": 18330 + i,
        })

    # Save JSON connection info
    with open(output_dir / "oracle-cloud-connection.json", "w") as f:
        json.dump(connection_info, f, indent=2)

    print(f"\nConnection info saved to: {output_dir / 'oracle-cloud-connection.json'}")
    print(f"Environment files saved to: {output_dir}/node*-env.sh")

    # Generate SSH command
    print("\n" + "=" * 60)
    print("  NEXT STEPS")
    print("=" * 60)
    print("""
1. Transfer binary to each node:
   scp -i ~/.ssh/id_rsa ./Udayad ubuntu@<IP>:/tmp/Udayad

2. Transfer deployment scripts:
   scp -i ~/.ssh/id_rsa deployments/cloud/* ubuntu@<IP>:/tmp/

3. Deploy each node:
   ssh -i ~/.ssh/id_rsa ubuntu@<IP> "sudo bash /tmp/deploy-node.sh <1|2|3> <rpc-password>"

4. Verify:
   ssh -i ~/.ssh/id_rsa ubuntu@<IP> "systemctl status udaya-node@1.service"
   ssh -i ~/.ssh/id_rsa ubuntu@<IP> "journalctl -u udaya-node@1.service -f"
""")


if __name__ == "__main__":
    main()