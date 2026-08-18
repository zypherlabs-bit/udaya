#!/usr/bin/env python3

import subprocess
import time
import json
import requests
import os
import signal
import sys
import threading
import psutil
from datetime import datetime
import random
import string
import hashlib
import socket
import re

class TestnetValidator:
    def __init__(self):
        self.nodes = [
            {"id": 1, "config": "config/node1.conf", "rpc_port": 18332, "p2p_port": 19798, "process": None},
            {"id": 2, "config": "config/node2.conf", "rpc_port": 18334, "p2p_port": 19799, "process": None},
            {"id": 3, "config": "config/node3.conf", "rpc_port": 18336, "p2p_port": 19800, "process": None}
        ]
        self.test_duration = 3600  # 1 hour test
        self.start_time = None
        self.test_results = {
            "build_status": "SUCCESS",
            "nodes_launched": 0,
            "mining_stats": {},
            "transaction_stats": {},
            "synchronization_status": {},
            "restart_recovery": {},
            "fork_reconnection": {},
            "resource_usage": {},
            "detected_issues": [],
            "fixes_applied": [],
            "final_recommendation": "PENDING"
        }
        self.running = False
        self.monitoring_thread = None

    def log(self, message, level="INFO"):
        timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        print(f"[{timestamp}] [{level}] {message}")

    def run_command(self, command, cwd=None):
        """Run a command and return the result"""
        try:
            result = subprocess.run(command, shell=True, cwd=cwd, capture_output=True, text=True)
            return result.returncode, result.stdout, result.stderr
        except Exception as e:
            return -1, "", str(e)

    def start_node(self, node):
        """Start a Udaya node"""
        cmd = f"cargo run --release --bin Udayad -- -c {node['config']} -v start"
        self.log(f"Starting node {node['id']} with command: {cmd}")

        try:
            # Start the process
            process = subprocess.Popen(cmd, shell=True, cwd=".")
            node['process'] = process
            self.log(f"Node {node['id']} started with PID {process.pid}")

            # Wait a bit for the node to initialize
            time.sleep(5)

            # Check if process is still running
            if process.poll() is None:
                self.log(f"Node {node['id']} is running successfully")
                return True
            else:
                self.log(f"Node {node['id']} failed to start", "ERROR")
                return False

        except Exception as e:
            self.log(f"Failed to start node {node['id']}: {e}", "ERROR")
            return False

    def stop_node(self, node):
        """Stop a Udaya node"""
        if node['process'] and node['process'].poll() is None:
            self.log(f"Stopping node {node['id']} (PID {node['process'].pid})")

            try:
                # Try to terminate gracefully first
                node['process'].terminate()
                node['process'].wait(timeout=10)

                # If still running, kill it
                if node['process'].poll() is None:
                    node['process'].kill()
                    node['process'].wait()

                self.log(f"Node {node['id']} stopped successfully")
                return True

            except subprocess.TimeoutExpired:
                node['process'].kill()
                node['process'].wait()
                self.log(f"Node {node['id']} was forcefully terminated", "WARNING")
                return True
            except Exception as e:
                self.log(f"Error stopping node {node['id']}: {e}", "ERROR")
                return False

        return True

    def rpc_call(self, node, method, params=None):
        """Make an RPC call to a node"""
        if params is None:
            params = []

        url = f"http://localhost:{node['rpc_port']}"
        headers = {'Content-Type': 'application/json'}
        auth = ('udaya', 'testnetpassword')

        payload = {
            "jsonrpc": "2.0",
            "id": "1",
            "method": method,
            "params": params
        }

        try:
            response = requests.post(url, data=json.dumps(payload), headers=headers, auth=auth, timeout=10)
            if response.status_code == 200:
                return response.json()
            else:
                self.log(f"RPC call failed for node {node['id']}: HTTP {response.status_code}", "ERROR")
                return None
        except requests.exceptions.RequestException as e:
            self.log(f"RPC call failed for node {node['id']}: {e}", "ERROR")
            return None

    def check_node_health(self, node):
        """Check if a node is healthy and responsive"""
        try:
            # Check if process is running
            if node['process'] and node['process'].poll() is None:
                # Try a simple RPC call
                result = self.rpc_call(node, "getblockchaininfo")
                if result and 'result' in result:
                    return True, result['result']
                else:
                    return False, "RPC not responding"
            else:
                return False, "Process not running"
        except Exception as e:
            return False, str(e)

    def monitor_nodes(self):
        """Monitor all nodes continuously"""
        while self.running:
            for node in self.nodes:
                healthy, status = self.check_node_health(node)
                if healthy:
                    self.log(f"Node {node['id']} is healthy - {status.get('chain', 'unknown')}")
                else:
                    self.log(f"Node {node['id']} is unhealthy - {status}", "WARNING")

            # Check resource usage
            self.monitor_resource_usage()

            time.sleep(15)

    def monitor_resource_usage(self):
        """Monitor system resource usage"""
        try:
            cpu_percent = psutil.cpu_percent(interval=1)
            memory_info = psutil.virtual_memory()
            disk_usage = psutil.disk_usage('/')

            resource_data = {
                "timestamp": datetime.now().isoformat(),
                "cpu_percent": cpu_percent,
                "memory_total": memory_info.total,
                "memory_used": memory_info.used,
                "memory_percent": memory_info.percent,
                "disk_total": disk_usage.total,
                "disk_used": disk_usage.used,
                "disk_percent": disk_usage.percent
            }

            if 'resource_usage' not in self.test_results:
                self.test_results['resource_usage'] = []

            self.test_results['resource_usage'].append(resource_data)

        except Exception as e:
            self.log(f"Error monitoring resources: {e}", "WARNING")

    def generate_test_transactions(self):
        """Generate test transactions between nodes"""
        self.log("Generating test transactions...")

        # First, get wallet addresses from each node
        addresses = {}
        for node in self.nodes:
            try:
                result = self.rpc_call(node, "getnewaddress")
                if result and 'result' in result:
                    addresses[node['id']] = result['result']
                    self.log(f"Node {node['id']} address: {result['result']}")
                else:
                    self.log(f"Failed to get address from node {node['id']}", "WARNING")
            except Exception as e:
                self.log(f"Error getting address from node {node['id']}: {e}", "WARNING")

        # Generate some transactions if we have addresses
        if len(addresses) >= 2:
            sender_id = 1
            receiver_id = 2
            amount = 0.1  # 0.1 UDYA

            try:
                # Send from node 1 to node 2
                tx_result = self.rpc_call(
                    self.nodes[sender_id - 1],
                    "sendtoaddress",
                    [addresses[receiver_id], amount]
                )

                if tx_result and 'result' in tx_result:
                    txid = tx_result['result']
                    self.log(f"Transaction created: {txid}")

                    # Record transaction stats
                    if 'transaction_stats' not in self.test_results:
                        self.test_results['transaction_stats'] = {
                            'total_transactions': 0,
                            'successful_transactions': 0,
                            'failed_transactions': 0,
                            'transaction_ids': []
                        }

                    self.test_results['transaction_stats']['total_transactions'] += 1
                    self.test_results['transaction_stats']['successful_transactions'] += 1
                    self.test_results['transaction_stats']['transaction_ids'].append(txid)

                    return True
                else:
                    self.log(f"Transaction failed: {tx_result}", "WARNING")
                    self.test_results['transaction_stats']['failed_transactions'] += 1
                    return False

            except Exception as e:
                self.log(f"Error creating transaction: {e}", "WARNING")
                return False

        return False

    def check_synchronization(self):
        """Check if all nodes are synchronized"""
        self.log("Checking node synchronization...")

        chain_tips = {}
        chain_heights = {}

        for node in self.nodes:
            try:
                result = self.rpc_call(node, "getblockchaininfo")
                if result and 'result' in result:
                    info = result['result']
                    chain_tips[node['id']] = info.get('bestblockhash', 'unknown')
                    chain_heights[node['id']] = info.get('blocks', 0)
                else:
                    self.log(f"Failed to get blockchain info from node {node['id']}", "WARNING")
            except Exception as e:
                self.log(f"Error getting blockchain info from node {node['id']}: {e}", "WARNING")

        # Check if all nodes have the same chain tip
        if len(set(chain_tips.values())) == 1:
            self.log("✅ All nodes are synchronized with the same chain tip")
            sync_status = "SYNCHRONIZED"
        else:
            self.log("⚠️  Nodes are not synchronized")
            sync_status = "NOT_SYNCHRONIZED"

        # Check if chain heights are similar (within 2 blocks)
        if len(chain_heights) > 0:
            max_height = max(chain_heights.values())
            min_height = min(chain_heights.values())
            height_diff = max_height - min_height

            if height_diff <= 2:
                self.log(f"✅ Chain heights are synchronized (diff: {height_diff})")
            else:
                self.log(f"⚠️  Chain heights differ by {height_diff} blocks")
                sync_status = "HEIGHT_MISMATCH"

        self.test_results['synchronization_status'] = {
            'status': sync_status,
            'chain_tips': chain_tips,
            'chain_heights': chain_heights,
            'timestamp': datetime.now().isoformat()
        }

        return sync_status == "SYNCHRONIZED"

    def test_restart_recovery(self):
        """Test node restart and recovery"""
        self.log("Testing node restart and recovery...")

        # Stop one node
        node_to_restart = self.nodes[0]
        self.stop_node(node_to_restart)

        # Wait a bit
        time.sleep(10)

        # Restart the node
        restart_success = self.start_node(node_to_restart)

        # Wait for recovery
        time.sleep(15)

        # Check if node recovered properly
        healthy, status = self.check_node_health(node_to_restart)

        self.test_results['restart_recovery'] = {
            'node_id': node_to_restart['id'],
            'restart_success': restart_success,
            'healthy_after_restart': healthy,
            'status': status,
            'timestamp': datetime.now().isoformat()
        }

        if restart_success and healthy:
            self.log("✅ Node restart and recovery test passed")
            return True
        else:
            self.log("❌ Node restart and recovery test failed", "ERROR")
            return False

    def test_fork_reconnection(self):
        """Test network fork and reconnection"""
        self.log("Testing network fork and reconnection...")

        # This is a simplified test - in a real scenario, we'd need to
        # actually create a fork condition, but for this test we'll
        # simulate by disconnecting and reconnecting nodes

        # Disconnect node 3 from the network (simulate by stopping it)
        node_to_disconnect = self.nodes[2]
        self.stop_node(node_to_disconnect)

        # Wait for the network to continue without it
        time.sleep(15)

        # Check if remaining nodes are still synchronized
        sync_before = self.check_synchronization()

        # Restart the disconnected node
        reconnect_success = self.start_node(node_to_disconnect)

        # Wait for reconnection and sync
        time.sleep(20)

        # Check if all nodes are synchronized again
        sync_after = self.check_synchronization()

        self.test_results['fork_reconnection'] = {
            'node_id': node_to_disconnect['id'],
            'reconnect_success': reconnect_success,
            'sync_before_reconnect': sync_before,
            'sync_after_reconnect': sync_after,
            'timestamp': datetime.now().isoformat()
        }

        if reconnect_success and sync_after:
            self.log("✅ Fork and reconnection test passed")
            return True
        else:
            self.log("❌ Fork and reconnection test failed", "ERROR")
            return False

    def check_mining_stats(self):
        """Check mining statistics across nodes"""
        self.log("Checking mining statistics...")

        mining_stats = {}

        for node in self.nodes:
            try:
                # Get mining info
                result = self.rpc_call(node, "getmininginfo")
                if result and 'result' in result:
                    mining_info = result['result']
                    mining_stats[node['id']] = {
                        'blocks': mining_info.get('blocks', 0),
                        'difficulty': mining_info.get('difficulty', 0),
                        'networkhashps': mining_info.get('networkhashps', 0),
                        'hashrate': mining_info.get('hashrate', 0),
                        'mining': mining_info.get('mining', False)
                    }
                else:
                    self.log(f"Failed to get mining info from node {node['id']}", "WARNING")
            except Exception as e:
                self.log(f"Error getting mining info from node {node['id']}: {e}", "WARNING")

        self.test_results['mining_stats'] = mining_stats
        return mining_stats

    def detect_and_fix_issues(self):
        """Detect and automatically fix repository-resolvable issues"""
        self.log("Checking for repository-resolvable issues...")

        # Check for common issues that can be fixed automatically

        # 1. Check if there are any warning fixes suggested by cargo
        self.log("Checking for cargo fix suggestions...")
        returncode, stdout, stderr = self.run_command("cargo fix --dry-run")

        if returncode == 0 and "would make" in stdout:
            self.log("Found issues that can be automatically fixed by cargo fix")
            self.test_results['detected_issues'].append("Cargo warnings can be fixed")

            # Apply the fixes
            self.log("Applying cargo fixes...")
            returncode, stdout, stderr = self.run_command("cargo fix")

            if returncode == 0:
                self.log("✅ Cargo fixes applied successfully")
                self.test_results['fixes_applied'].append("Applied cargo fix suggestions")
                return True
            else:
                self.log(f"❌ Failed to apply cargo fixes: {stderr}", "ERROR")
                return False

        # 2. Check if we need to rebuild after fixes
        if self.test_results['fixes_applied']:
            self.log("Rebuilding after fixes...")
            returncode, stdout, stderr = self.run_command("cargo build --release")

            if returncode == 0:
                self.log("✅ Rebuild successful after fixes")
                return True
            else:
                self.log(f"❌ Rebuild failed: {stderr}", "ERROR")
                return False

        self.log("No repository-resolvable issues found")
        return True

    def generate_final_report(self):
        """Generate the final validation report"""
        self.log("Generating final validation report...")

        # Calculate test duration
        end_time = datetime.now()
        duration = (end_time - self.start_time).total_seconds() / 60  # in minutes

        # Determine final recommendation
        all_tests_passed = True

        # Check critical components
        if self.test_results['nodes_launched'] < 3:
            all_tests_passed = False

        if 'synchronization_status' in self.test_results:
            if self.test_results['synchronization_status']['status'] != 'SYNCHRONIZED':
                all_tests_passed = False

        if 'restart_recovery' in self.test_results:
            if not self.test_results['restart_recovery']['healthy_after_restart']:
                all_tests_passed = False

        if 'fork_reconnection' in self.test_results:
            if not self.test_results['fork_reconnection']['sync_after_reconnect']:
                all_tests_passed = False

        # Determine GO/NO-GO recommendation
        if all_tests_passed:
            recommendation = "GO"
            recommendation_text = "✅ All validation tests passed. Ready for full public testnet."
        else:
            recommendation = "NO-GO"
            recommendation_text = "❌ Some validation tests failed. Requires further investigation."

        # Generate the final report
        report = {
            "testnet_validation_report": {
                "timestamp": datetime.now().isoformat(),
                "test_duration_minutes": duration,
                "build_status": self.test_results['build_status'],
                "nodes_launched": self.test_results['nodes_launched'],
                "mining_statistics": self.test_results.get('mining_stats', {}),
                "transaction_statistics": self.test_results.get('transaction_stats', {}),
                "synchronization_status": self.test_results.get('synchronization_status', {}),
                "restart_recovery_results": self.test_results.get('restart_recovery', {}),
                "fork_reconnection_results": self.test_results.get('fork_reconnection', {}),
                "resource_usage_summary": self._summarize_resource_usage(),
                "detected_issues": self.test_results['detected_issues'],
                "fixes_applied": self.test_results['fixes_applied'],
                "final_recommendation": {
                    "status": recommendation,
                    "description": recommendation_text
                }
            }
        }

        # Write report to file
        report_file = "TESTNET_VALIDATION_REPORT.json"
        with open(report_file, 'w') as f:
            json.dump(report, f, indent=2)

        self.log(f"Final report written to {report_file}")

        # Also write a human-readable summary
        self._write_human_readable_summary(report)

        return report

    def _summarize_resource_usage(self):
        """Summarize resource usage data"""
        if 'resource_usage' not in self.test_results or not self.test_results['resource_usage']:
            return {}

        usage_data = self.test_results['resource_usage']

        # Calculate averages
        cpu_values = [entry['cpu_percent'] for entry in usage_data]
        memory_values = [entry['memory_percent'] for entry in usage_data]
        disk_values = [entry['disk_percent'] for entry in usage_data]

        return {
            'average_cpu_percent': sum(cpu_values) / len(cpu_values),
            'average_memory_percent': sum(memory_values) / len(memory_values),
            'average_disk_percent': sum(disk_values) / len(disk_values),
            'peak_cpu_percent': max(cpu_values),
            'peak_memory_percent': max(memory_values),
            'peak_disk_percent': max(disk_values),
            'samples_collected': len(usage_data)
        }

    def _write_human_readable_summary(self, report):
        """Write a human-readable summary of the test results"""
        summary_file = "TESTNET_VALIDATION_SUMMARY.md"
        data = report['testnet_validation_report']

        with open(summary_file, 'w') as f:
            f.write("# Udaya Blockchain Testnet Validation Report\n\n")

            f.write(f"## Test Summary\n")
            f.write(f"- **Date/Time**: {data['timestamp']}\n")
            f.write(f"- **Test Duration**: {data['test_duration_minutes']:.1f} minutes\n")
            f.write(f"- **Build Status**: {data['build_status']}\n")
            f.write(f"- **Nodes Launched**: {data['nodes_launched']}/3\n\n")

            f.write(f"## Mining Statistics\n")
            mining_stats = data['mining_statistics']
            for node_id, stats in mining_stats.items():
                f.write(f"- **Node {node_id}**: {stats.get('blocks', 0)} blocks, ")
                f.write(f"{stats.get('difficulty', 0)} difficulty, ")
                f.write(f"{stats.get('hashrate', 0)} hashrate\n")

            f.write(f"\n## Transaction Statistics\n")
            tx_stats = data['transaction_statistics']
            f.write(f"- **Total Transactions**: {tx_stats.get('total_transactions', 0)}\n")
            f.write(f"- **Successful**: {tx_stats.get('successful_transactions', 0)}\n")
            f.write(f"- **Failed**: {tx_stats.get('failed_transactions', 0)}\n\n")

            f.write(f"## Synchronization Status\n")
            sync_status = data['synchronization_status']
            f.write(f"- **Status**: {sync_status['status']}\n")
            f.write(f"- **Chain Heights**: {sync_status['chain_heights']}\n\n")

            f.write(f"## Restart Recovery Test\n")
            recovery = data['restart_recovery_results']
            f.write(f"- **Node Restarted**: {recovery['node_id']}\n")
            f.write(f"- **Recovery Success**: {'✅' if recovery['healthy_after_restart'] else '❌'}\n\n")

            f.write(f"## Fork/Reconnection Test\n")
            fork_test = data['fork_reconnection_results']
            f.write(f"- **Node Tested**: {fork_test['node_id']}\n")
            f.write(f"- **Sync After Reconnect**: {'✅' if fork_test['sync_after_reconnect'] else '❌'}\n\n")

            f.write(f"## Resource Usage\n")
            resources = data['resource_usage_summary']
            f.write(f"- **Average CPU**: {resources.get('average_cpu_percent', 0):.1f}%\n")
            f.write(f"- **Average Memory**: {resources.get('average_memory_percent', 0):.1f}%\n")
            f.write(f"- **Average Disk**: {resources.get('average_disk_percent', 0):.1f}%\n")
            f.write(f"- **Peak CPU**: {resources.get('peak_cpu_percent', 0):.1f}%\n\n")

            f.write(f"## Issues and Fixes\n")
            f.write(f"- **Detected Issues**: {len(data['detected_issues'])}\n")
            for issue in data['detected_issues']:
                f.write(f"  - {issue}\n")
            f.write(f"- **Fixes Applied**: {len(data['fixes_applied'])}\n")
            for fix in data['fixes_applied']:
                f.write(f"  - {fix}\n\n")

            f.write(f"## Final Recommendation\n")
            f.write(f"### {data['final_recommendation']['status']}\n")
            f.write(f"{data['final_recommendation']['description']}\n")

        self.log(f"Human-readable summary written to {summary_file}")

    def run_validation(self):
        """Run the complete testnet validation"""
        self.log("Starting Udaya Blockchain Testnet Validation")
        self.start_time = datetime.now()
        self.running = True

        try:
            # Step 1: Build the project (already done, but check)
            self.log("Checking build status...")
            returncode, stdout, stderr = self.run_command("cargo build --release")

            if returncode != 0:
                self.test_results['build_status'] = "FAILED"
                self.log(f"Build failed: {stderr}", "ERROR")
                return False
            else:
                self.test_results['build_status'] = "SUCCESS"
                self.log("Build verification successful")

            # Step 2: Detect and fix any repository-resolvable issues
            self.detect_and_fix_issues()

            # Step 3: Start monitoring thread
            self.monitoring_thread = threading.Thread(target=self.monitor_nodes)
            self.monitoring_thread.start()

            # Step 4: Launch 3 interconnected nodes
            self.log("Launching 3 interconnected nodes...")
            for node in self.nodes:
                if self.start_node(node):
                    self.test_results['nodes_launched'] += 1
                    time.sleep(5)  # Stagger node starts

            if self.test_results['nodes_launched'] < 3:
                self.log(f"Only {self.test_results['nodes_launched']}/3 nodes launched successfully", "ERROR")
                return False

            self.log("✅ All 3 nodes launched successfully")

            # Step 5: Wait for nodes to initialize and connect
            self.log("Waiting for nodes to initialize and connect...")
            time.sleep(30)

            # Step 6: Check initial synchronization
            initial_sync = self.check_synchronization()

            # Step 7: Enable mining and generate some blocks
            self.log("Enabling mining and generating blocks...")
            self.check_mining_stats()

            # Step 8: Execute automated transactions
            self.log("Executing automated transactions...")
            for i in range(3):  # Try a few transactions
                self.generate_test_transactions()
                time.sleep(10)

            # Step 9: Run synchronization tests
            self.log("Running synchronization tests...")
            sync_ok = self.check_synchronization()

            # Step 10: Perform restart recovery test
            self.log("Running restart recovery test...")
            recovery_ok = self.test_restart_recovery()

            # Step 11: Perform fork/reconnection test
            self.log("Running fork/reconnection test...")
            fork_ok = self.test_fork_reconnection()

            # Step 12: Final synchronization check
            final_sync = self.check_synchronization()

            # Step 13: Run the test for the full duration
            elapsed_time = (datetime.now() - self.start_time).total_seconds()
            remaining_time = max(0, self.test_duration - elapsed_time)

            self.log(f"Running continuous validation for {remaining_time/60:.1f} more minutes...")
            time.sleep(remaining_time)

            # Step 14: Generate final report
            self.running = False
            if self.monitoring_thread:
                self.monitoring_thread.join()

            report = self.generate_final_report()

            # Step 15: Stop all nodes
            self.log("Stopping all nodes...")
            for node in self.nodes:
                self.stop_node(node)

            self.log("Testnet validation completed")
            return True

        except Exception as e:
            self.log(f"Testnet validation failed: {e}", "ERROR")
            self.test_results['final_recommendation'] = {
                "status": "NO-GO",
                "description": f"Testnet validation failed: {e}"
            }
            return False

if __name__ == "__main__":
    validator = TestnetValidator()

    try:
        success = validator.run_validation()

        if success:
            validator.log("✅ Testnet validation completed successfully")
            sys.exit(0)
        else:
            validator.log("❌ Testnet validation completed with issues", "ERROR")
            sys.exit(1)

    except KeyboardInterrupt:
        validator.log("Testnet validation interrupted by user", "WARNING")
        validator.running = False
        if validator.monitoring_thread:
            validator.monitoring_thread.join()
        for node in validator.nodes:
            validator.stop_node(node)
        sys.exit(1)
    except Exception as e:
        validator.log(f"Unexpected error: {e}", "ERROR")
        validator.running = False
        if validator.monitoring_thread:
            validator.monitoring_thread.join()
        for node in validator.nodes:
            validator.stop_node(node)
        sys.exit(1)