// =============================================
// Udaya Ecosystem - Premium Financial Infrastructure
// "Decentralized Financial Infrastructure"
// Interactive 3D Globe, Network Visualization, Premium Animations
// =============================================

const Udaya = {
  version: '2.0.0',
  
  // API endpoints
  API: {
    stats: '/api/stats',
    nodes: '/api/nodes',
    blocks: '/api/blocks',
    transactions: '/api/transactions',
    register: '/api/register',
    verify: '/api/verify',
    rewards: '/api/rewards',
    metrics: '/api/metrics',
  },

  // =============================================
  // Check for reduced motion preferences
  // =============================================
  
  prefersReducedMotion() {
    return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  },

  // =============================================
  // 3D Globe using Three.js (CDN)
  // =============================================
  
  initThreeGlobe() {
    // This will be called if Three.js is loaded
    if (typeof THREE === 'undefined' || this.prefersReducedMotion()) return;
    
    const container = document.getElementById('globe-container');
    if (!container) return;

    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(45, window.innerWidth / window.innerHeight, 0.1, 1000);
    const renderer = new THREE.WebGLRenderer({ 
      alpha: true, 
      antialias: true,
      powerPreference: "high-performance"
    });
    
    renderer.setSize(window.innerWidth, window.innerHeight);
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    container.appendChild(renderer.domElement);

    // Create globe
    const globeGeometry = new THREE.SphereGeometry(3, 64, 64);
    const globeMaterial = new THREE.MeshPhongMaterial({
      color: 0x1A2332,
      emissive: 0x0B1120,
      emissiveIntensity: 0.3,
      wireframe: false,
      transparent: true,
      opacity: 0.9,
    });
    const globe = new THREE.Mesh(globeGeometry, globeMaterial);
    scene.add(globe);

    // Gold wireframe overlay
    const wireframeGeometry = new THREE.SphereGeometry(3.05, 32, 32);
    const wireframeMaterial = new THREE.MeshBasicMaterial({
      color: 0xF59E0B,
      wireframe: true,
      transparent: true,
      opacity: 0.08,
    });
    const wireframe = new THREE.Mesh(wireframeGeometry, wireframeMaterial);
    scene.add(wireframe);

    // Node markers on globe
    const nodePositions = [
      { lat: 40.7128, lng: -74.0060, color: 0xF59E0B, size: 0.08 },  // NYC
      { lat: 51.5074, lng: -0.1278, color: 0x00E5FF, size: 0.07 },  // London
      { lat: 35.6762, lng: 139.6503, color: 0x7C3AED, size: 0.07 }, // Tokyo
      { lat: -33.8688, lng: 151.2093, color: 0x10B981, size: 0.06 }, // Sydney
      { lat: 1.3521, lng: 103.8198, color: 0xF59E0B, size: 0.06 },  // Singapore
      { lat: 55.7558, lng: 37.6173, color: 0x00E5FF, size: 0.05 },  // Moscow
      { lat: -23.5505, lng: -46.6333, color: 0x7C3AED, size: 0.05 }, // Sao Paulo
      { lat: 28.6139, lng: 77.2090, color: 0x10B981, size: 0.06 },  // Delhi
      { lat: 31.2304, lng: 121.4737, color: 0xF59E0B, size: 0.07 }, // Shanghai
      { lat: 48.8566, lng: 2.3522, color: 0x00E5FF, size: 0.05 },   // Paris
      { lat: 37.7749, lng: -122.4194, color: 0x7C3AED, size: 0.06 }, // SF
      { lat: 25.0343, lng: 121.5645, color: 0x10B981, size: 0.05 }, // Taipei
    ];

    const nodes = [];
    const connectionLines = [];

    nodePositions.forEach((pos, i) => {
      const phi = (90 - pos.lat) * Math.PI / 180;
      const theta = pos.lng * Math.PI / 180;
      
      const x = 3 * Math.sin(phi) * Math.cos(theta);
      const y = 3 * Math.cos(phi);
      const z = 3 * Math.sin(phi) * Math.sin(theta);

      // Node sphere
      const nodeGeo = new THREE.SphereGeometry(pos.size, 8, 8);
      const nodeMat = new THREE.MeshBasicMaterial({ color: pos.color });
      const node = new THREE.Mesh(nodeGeo, nodeMat);
      node.position.set(x, y, z);
      scene.add(node);
      nodes.push(node);

      // Node glow
      const glowGeo = new THREE.SphereGeometry(pos.size * 3, 8, 8);
      const glowMat = new THREE.MeshBasicMaterial({
        color: pos.color,
        transparent: true,
        opacity: 0.15,
      });
      const glow = new THREE.Mesh(glowGeo, glowMat);
      glow.position.set(x, y, z);
      scene.add(glow);

      // Connect to a few random nodes
      const connections = Math.floor(Math.random() * 3) + 1;
      for (let c = 0; c < connections; c++) {
        const j = Math.floor(Math.random() * nodePositions.length);
        if (j !== i) {
          const p2 = nodePositions[j];
          const phi2 = (90 - p2.lat) * Math.PI / 180;
          const theta2 = p2.lng * Math.PI / 180;
          const x2 = 3 * Math.sin(phi2) * Math.cos(theta2);
          const y2 = 3 * Math.cos(phi2);
          const z2 = 3 * Math.sin(phi2) * Math.sin(theta2);

          const curvePoints = [];
          const midX = (x + x2) / 2;
          const midY = (y + y2) / 2;
          const midZ = (z + z2) / 2;
          const midLen = Math.sqrt(midX*midX + midY*midY + midZ*midZ);
          const arcHeight = 1.5;
          const arcX = midX / midLen * (3 + arcHeight);
          const arcY = midY / midLen * (3 + arcHeight);
          const arcZ = midZ / midLen * (3 + arcHeight);

          const curve = new THREE.QuadraticBezierCurve3(
            new THREE.Vector3(x, y, z),
            new THREE.Vector3(arcX, arcY, arcZ),
            new THREE.Vector3(x2, y2, z2)
          );

          const curveGeo = new THREE.BufferGeometry().setFromPoints(curve.getPoints(20));
          const edgeColor = new THREE.Color(pos.color).lerp(new THREE.Color(p2.color), 0.5);
          const curveMat = new THREE.LineBasicMaterial({
            color: edgeColor,
            transparent: true,
            opacity: 0.12,
            linewidth: 1,
          });
          const curveLine = new THREE.Line(curveGeo, curveMat);
          scene.add(curveLine);
          connectionLines.push(curveLine);
        }
      }
    });

    // Ambient light
    const ambientLight = new THREE.AmbientLight(0x222244, 0.5);
    scene.add(ambientLight);

    // Point lights
    const light1 = new THREE.PointLight(0xF59E0B, 1, 20);
    light1.position.set(5, 5, 5);
    scene.add(light1);

    const light2 = new THREE.PointLight(0x00E5FF, 0.5, 20);
    light2.position.set(-5, -3, -5);
    scene.add(light2);

    // Stars background
    const starsGeometry = new THREE.BufferGeometry();
    const starsCount = 600;
    const starsPositions = new Float32Array(starsCount * 3);
    for (let i = 0; i < starsCount * 3; i++) {
      starsPositions[i] = (Math.random() - 0.5) * 100;
    }
    starsGeometry.setAttribute('position', new THREE.BufferAttribute(starsPositions, 3));
    const starsMaterial = new THREE.PointsMaterial({
      color: 0xffffff,
      size: 0.05,
      transparent: true,
      opacity: 0.6,
    });
    const stars = new THREE.Points(starsGeometry, starsMaterial);
    scene.add(stars);

    camera.position.z = 8;

    // Mouse interaction
    let mouseX = 0;
    let mouseY = 0;
    
    document.addEventListener('mousemove', (e) => {
      mouseX = (e.clientX / window.innerWidth) * 2 - 1;
      mouseY = -(e.clientY / window.innerHeight) * 2 + 1;
    });

    // Animation loop
    const animate = () => {
      requestAnimationFrame(animate);

      // Rotate globe slowly
      globe.rotation.y += 0.0015;
      wireframe.rotation.y += 0.0015;
      stars.rotation.y -= 0.0003;

      // Rotate nodes
      nodes.forEach(node => {
        node.rotation.x += 0.01;
        node.rotation.y += 0.01;
      });

      // Follow mouse gently
      globe.rotation.x += (mouseY * 0.1 - globe.rotation.x) * 0.02;
      globe.rotation.y += (mouseX * 0.2 - globe.rotation.y) * 0.02;
      wireframe.rotation.x = globe.rotation.x;
      wireframe.rotation.y = globe.rotation.y;

      renderer.render(scene, camera);
    };

    animate();

    // Resize handler
    window.addEventListener('resize', () => {
      camera.aspect = window.innerWidth / window.innerHeight;
      camera.updateProjectionMatrix();
      renderer.setSize(window.innerWidth, window.innerHeight);
    });

    this._threeCleanup = () => {
      renderer.dispose();
      if (container.contains(renderer.domElement)) {
        container.removeChild(renderer.domElement);
      }
    };
  },

  // =============================================
  // Particle System - Floating Network Background
  // =============================================
  
  initParticles() {
    const canvas = document.getElementById('particle-canvas');
    if (!canvas || this.prefersReducedMotion()) return;

    const ctx = canvas.getContext('2d');
    let animationId;
    let particles = [];
    let mouseX = 0;
    let mouseY = 0;

    function resize() {
      canvas.width = window.innerWidth;
      canvas.height = window.innerHeight;
    }

    function createParticles(count = 150) {
      particles = [];
      for (let i = 0; i < count; i++) {
        particles.push({
          x: Math.random() * canvas.width,
          y: Math.random() * canvas.height,
          vx: (Math.random() - 0.5) * 0.4,
          vy: (Math.random() - 0.5) * 0.4,
          radius: Math.random() * 2 + 0.5,
          opacity: Math.random() * 0.4 + 0.1,
          hue: Math.random() > 0.6 ? 45 : Math.random() > 0.5 ? 190 : 260,
          pulse: Math.random() * Math.PI * 2,
        });
      }
    }

    function draw() {
      ctx.clearRect(0, 0, canvas.width, canvas.height);

      // Draw connections
      for (let i = 0; i < particles.length; i++) {
        for (let j = i + 1; j < particles.length; j++) {
          const dx = particles[i].x - particles[j].x;
          const dy = particles[i].y - particles[j].y;
          const dist = Math.sqrt(dx * dx + dy * dy);
          
          if (dist < 150) {
            ctx.beginPath();
            ctx.moveTo(particles[i].x, particles[i].y);
            ctx.lineTo(particles[j].x, particles[j].y);
            ctx.strokeStyle = `rgba(245, 158, 11, ${(1 - dist / 150) * 0.08})`;
            ctx.lineWidth = 0.5;
            ctx.stroke();
          }
        }
      }

      // Draw particles
      for (const p of particles) {
        p.pulse += 0.02;
        const pulseOpacity = 0.5 + 0.5 * Math.sin(p.pulse);
        
        // Mouse proximity effect
        const dx = p.x - mouseX;
        const dy = p.y - mouseY;
        const dist = Math.sqrt(dx * dx + dy * dy);
        const glowRadius = dist < 200 ? p.radius + (1 - dist / 200) * 3 : p.radius;

        ctx.beginPath();
        ctx.arc(p.x, p.y, glowRadius, 0, Math.PI * 2);
        
        const color = p.hue === 45 ? '245, 158, 11' : p.hue === 190 ? '0, 229, 255' : '124, 58, 237';
        const finalOpacity = p.opacity * (0.7 + 0.3 * pulseOpacity) + (dist < 200 ? (1 - dist / 200) * 0.3 : 0);
        ctx.fillStyle = `rgba(${color}, ${finalOpacity})`;
        ctx.fill();

        // Glow
        if (p.radius > 1.5) {
          ctx.beginPath();
          ctx.arc(p.x, p.y, glowRadius * 3, 0, Math.PI * 2);
          ctx.fillStyle = `rgba(${color}, ${0.04 * pulseOpacity})`;
          ctx.fill();
        }

        p.x += p.vx;
        p.y += p.vy;

        if (p.x < -10) p.x = canvas.width + 10;
        if (p.x > canvas.width + 10) p.x = -10;
        if (p.y < -10) p.y = canvas.height + 10;
        if (p.y > canvas.height + 10) p.y = -10;
      }

      animationId = requestAnimationFrame(draw);
    }

    window.addEventListener('resize', () => {
      resize();
      createParticles();
    });

    document.addEventListener('mousemove', (e) => {
      mouseX = e.clientX;
      mouseY = e.clientY;
    });

    resize();
    createParticles();
    draw();

    this._particleCleanup = () => {
      if (animationId) cancelAnimationFrame(animationId);
    };
  },

  // =============================================
  // Navigation - Premium Floating Glass Bar
  // =============================================
  
  initNav() {
    const toggle = document.querySelector('.nav-toggle');
    const navLinks = document.querySelector('.nav-links');
    const nav = document.querySelector('.nav');

    if (toggle && navLinks) {
      toggle.addEventListener('click', () => {
        const isOpen = navLinks.classList.toggle('open');
        toggle.setAttribute('aria-expanded', isOpen);
      });

      document.addEventListener('click', (e) => {
        if (!nav.contains(e.target) && navLinks.classList.contains('open')) {
          navLinks.classList.remove('open');
          toggle.setAttribute('aria-expanded', 'false');
        }
      });

      document.addEventListener('keydown', (e) => {
        if (e.key === 'Escape' && navLinks.classList.contains('open')) {
          navLinks.classList.remove('open');
          toggle.setAttribute('aria-expanded', 'false');
          toggle.focus();
        }
      });
    }

    let lastScroll = 0;
    window.addEventListener('scroll', () => {
      const scrollY = window.scrollY;
      if (scrollY > 50) {
        nav.classList.add('scrolled');
      } else {
        nav.classList.remove('scrolled');
      }
      lastScroll = scrollY;
    }, { passive: true });

    if (window.scrollY > 50) {
      nav.classList.add('scrolled');
    }

    const currentPath = window.location.pathname;
    document.querySelectorAll('.nav-links a').forEach(a => {
      const linkPath = a.getAttribute('href');
      if (linkPath && currentPath.startsWith(linkPath) && linkPath !== '/') {
        a.classList.add('active');
      } else if ((linkPath === '/' || linkPath === '/index.html') && 
                 (currentPath === '/' || currentPath === '' || currentPath === '/index.html')) {
        a.classList.add('active');
      }
    });
  },

  // =============================================
  // Intersection Observer - Scroll Reveal
  // =============================================
  
  initRevealAnimations() {
    if (this.prefersReducedMotion()) {
      document.querySelectorAll('.reveal').forEach(el => {
        el.classList.add('visible');
      });
      return;
    }

    const observer = new IntersectionObserver((entries) => {
      entries.forEach((entry) => {
        if (entry.isIntersecting) {
          setTimeout(() => {
            entry.target.classList.add('visible');
          }, 50);
          observer.unobserve(entry.target);
        }
      });
    }, { 
      threshold: 0.05,
      rootMargin: '0px 0px -50px 0px'
    });

    document.querySelectorAll('.reveal').forEach(el => {
      observer.observe(el);
    });

    this._revealObserver = observer;
  },

  // =============================================
  // Animated Counters - Premium
  // =============================================
  
  animateCounter(element, target, duration = 1500) {
    if (this.prefersReducedMotion()) {
      element.textContent = target.toLocaleString();
      return;
    }
    
    const start = 0;
    const startTime = performance.now();
    const isFloat = target % 1 !== 0;
    
    function update(currentTime) {
      const elapsed = currentTime - startTime;
      const progress = Math.min(elapsed / duration, 1);
      const eased = 1 - Math.pow(1 - progress, 3);
      const current = start + (target - start) * eased;
      
      element.textContent = isFloat ? current.toFixed(2) : Math.floor(current).toLocaleString();
      
      if (progress < 1) {
        requestAnimationFrame(update);
      } else {
        element.textContent = isFloat ? target.toFixed(2) : Math.floor(target).toLocaleString();
      }
    }
    
    requestAnimationFrame(update);
  },

  initCounters() {
    document.querySelectorAll('[data-animate-counter]').forEach(el => {
      const target = parseFloat(el.getAttribute('data-animate-counter')) || 0;
      const delay = parseInt(el.closest('.stat-card, .health-metric, .bento-card')?.className.match(/reveal-delay-(\d)/)?.[1] || '0') * 100;
      setTimeout(() => {
        this.animateCounter(el, target);
      }, delay + 300);
    });
  },

  // =============================================
  // Sparkline Charts
  // =============================================
  
  initSparklines() {
    const generateTrend = (points = 30, volatility = 0.3, startVal = 50) => {
      const data = [];
      let val = startVal;
      for (let i = 0; i < points; i++) {
        val += (Math.random() - 0.48) * volatility * startVal;
        val = Math.max(0, val);
        data.push(val);
      }
      return data;
    };

    const colors = [
      '#F59E0B', '#00E5FF', '#7C3AED', '#06B6D4',
      '#10B981', '#34D399', '#A78BFA', '#F59E0B'
    ];

    document.querySelectorAll('.stat-sparkline').forEach((canvas, index) => {
      if (!canvas) return;
      const ctx = canvas.getContext('2d');
      if (!ctx) return;
      
      const data = generateTrend(25, 0.25, 50 + Math.random() * 50);
      const w = canvas.width;
      const h = canvas.height;
      const min = Math.min(...data);
      const max = Math.max(...data);
      const range = max - min || 1;
      const color = colors[index % colors.length];

      function drawSparkline(dataPoints) {
        ctx.clearRect(0, 0, w, h);

        ctx.beginPath();
        ctx.strokeStyle = color;
        ctx.lineWidth = 1.5;
        ctx.lineJoin = 'round';
        ctx.lineCap = 'round';

        const dMin = Math.min(...dataPoints);
        const dMax = Math.max(...dataPoints);
        const dRange = dMax - dMin || 1;

        dataPoints.forEach((val, i) => {
          const x = (i / (dataPoints.length - 1)) * w;
          const y = h - ((val - dMin) / dRange) * (h - 6) - 3;
          i === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y);
        });
        ctx.stroke();

        const lastPoint = dataPoints.length - 1;
        const lastX = (lastPoint / (dataPoints.length - 1)) * w;
        const lastY = h - ((dataPoints[lastPoint] - dMin) / dRange) * (h - 6) - 3;

        ctx.lineTo(lastX, h);
        ctx.lineTo(0, h);
        ctx.closePath();

        const gradient = ctx.createLinearGradient(0, 0, 0, h);
        gradient.addColorStop(0, color + '25');
        gradient.addColorStop(1, color + '02');
        ctx.fillStyle = gradient;
        ctx.fill();
      }

      drawSparkline(data);

      setInterval(() => {
        const newData = generateTrend(25, 0.25, 50 + Math.random() * 50);
        drawSparkline(newData);
      }, 5000 + Math.random() * 3000);
    });
  },

  // =============================================
  // Network Visualization (2D Canvas)
  // =============================================
  
  initNetworkViz() {
    const canvas = document.getElementById('network-canvas');
    if (!canvas || this.prefersReducedMotion()) return;

    const ctx = canvas.getContext('2d');
    const rect = canvas.parentElement.getBoundingClientRect();
    canvas.width = rect.width;
    canvas.height = rect.height;

    // Create nodes
    const nodes = [];
    const nodeLabels = ['Node A', 'Node B', 'Node C', 'Node D', 'Node E', 'Node F'];
    const colors = ['#F59E0B', '#00E5FF', '#7C3AED', '#10B981', '#F59E0B', '#00E5FF'];
    
    for (let i = 0; i < 6; i++) {
      const angle = (i / 6) * Math.PI * 2 - Math.PI / 2;
      const radius = 80 + Math.random() * 40;
      nodes.push({
        x: canvas.width / 2 + Math.cos(angle) * radius,
        y: canvas.height / 2 + Math.sin(angle) * radius,
        vx: (Math.random() - 0.5) * 0.3,
        vy: (Math.random() - 0.5) * 0.3,
        radius: 8 + Math.random() * 4,
        color: colors[i],
        label: nodeLabels[i],
        pulse: Math.random() * Math.PI * 2,
      });
    }

    // Connections
    const connections = [
      [0, 1], [1, 2], [2, 3], [3, 4], [4, 5], [5, 0],
      [0, 2], [1, 3], [2, 4], [3, 5], [4, 0], [5, 1]
    ];

    // Transaction pulse points
    let pulseProgress = 0;
    let currentConnection = 0;

    function animate() {
      ctx.clearRect(0, 0, canvas.width, canvas.height);

      // Draw connections
      connections.forEach(([i, j], idx) => {
        const nodeA = nodes[i];
        const nodeB = nodes[j];
        
        ctx.beginPath();
        ctx.moveTo(nodeA.x, nodeA.y);
        ctx.lineTo(nodeB.x, nodeB.y);
        ctx.strokeStyle = `rgba(245, 158, 11, 0.1)`;
        ctx.lineWidth = 1;
        ctx.stroke();

        // Pulse on active connections
        if (idx === currentConnection) {
          const px = nodeA.x + (nodeB.x - nodeA.x) * pulseProgress;
          const py = nodeA.y + (nodeB.y - nodeA.y) * pulseProgress;

          ctx.beginPath();
          ctx.arc(px, py, 4, 0, Math.PI * 2);
          ctx.fillStyle = '#F59E0B';
          ctx.fill();

          ctx.beginPath();
          ctx.arc(px, py, 8, 0, Math.PI * 2);
          ctx.fillStyle = 'rgba(245, 158, 11, 0.2)';
          ctx.fill();

          ctx.strokeStyle = `rgba(245, 158, 11, ${0.3 * (1 - pulseProgress)})`;
          ctx.lineWidth = 2;
          ctx.stroke();

          // Glow line
          ctx.strokeStyle = `rgba(245, 158, 11, ${0.15 * (1 - pulseProgress)})`;
          ctx.lineWidth = 4;
          ctx.stroke();
        }
      });

      // Draw nodes
      nodes.forEach((node, i) => {
        node.pulse += 0.03;
        const pulseR = Math.sin(node.pulse) * 0.3 + 1;

        // Glow
        const gradient = ctx.createRadialGradient(node.x, node.y, 0, node.x, node.y, node.radius * 4);
        gradient.addColorStop(0, node.color + '40');
        gradient.addColorStop(1, node.color + '00');
        ctx.fillStyle = gradient;
        ctx.beginPath();
        ctx.arc(node.x, node.y, node.radius * 4, 0, Math.PI * 2);
        ctx.fill();

        // Node circle
        ctx.beginPath();
        ctx.arc(node.x, node.y, node.radius * pulseR, 0, Math.PI * 2);
        ctx.fillStyle = node.color;
        ctx.fill();

        // Node border
        ctx.strokeStyle = node.color + '40';
        ctx.lineWidth = 2;
        ctx.stroke();

        // Label
        ctx.fillStyle = '#8BA0C8';
        ctx.font = '11px Inter, sans-serif';
        ctx.textAlign = 'center';
        ctx.fillText(node.label, node.x, node.y - node.radius - 12);

        // Move nodes slightly
        node.x += node.vx;
        node.y += node.vy;

        // Keep in bounds
        const cx = canvas.width / 2;
        const cy = canvas.height / 2;
        const dx = node.x - cx;
        const dy = node.y - cy;
        const dist = Math.sqrt(dx * dx + dy * dy);
        if (dist > 150) {
          node.vx -= dx * 0.001;
          node.vy -= dy * 0.001;
        }
      });

      // Update pulse
      pulseProgress += 0.02;
      if (pulseProgress >= 1) {
        pulseProgress = 0;
        currentConnection = (currentConnection + 1) % connections.length;
      }

      requestAnimationFrame(animate);
    }

    animate();

    // Resize
    window.addEventListener('resize', () => {
      const newRect = canvas.parentElement.getBoundingClientRect();
      canvas.width = newRect.width;
      canvas.height = newRect.height;
    });
  },

  // =============================================
  // Scroll Progress Indicator
  // =============================================
  
  initScrollProgress() {
    const bar = document.getElementById('scroll-progress');
    if (!bar) return;

    window.addEventListener('scroll', () => {
      const scrollTop = document.documentElement.scrollTop || document.body.scrollTop;
      const scrollHeight = document.documentElement.scrollHeight - document.documentElement.clientHeight;
      const progress = (scrollTop / scrollHeight) * 100;
      bar.style.width = progress + '%';
    }, { passive: true });
  },

  // =============================================
  // Live Data Simulation for Metrics
  // =============================================
  
  initLiveMetrics() {
    // Update "last updated" timestamps
    document.querySelectorAll('[data-live-time]').forEach(el => {
      const updateTime = () => {
        const now = new Date();
        el.textContent = now.toLocaleTimeString('en-US', { 
          hour: '2-digit', 
          minute: '2-digit', 
          second: '2-digit' 
        });
      };
      updateTime();
      setInterval(updateTime, 1000);
    });
  },

  // =============================================
  // Parallax Effect on Hero
  // =============================================
  
  initParallax() {
    if (this.prefersReducedMotion()) return;
    
    window.addEventListener('scroll', () => {
      const scrolled = window.pageYOffset;
      const hero = document.querySelector('.hero');
      const globe = document.querySelector('.hero-globe-container');
      
      if (hero && globe) {
        globe.style.transform = `translate(-50%, calc(-50% + ${scrolled * 0.15}px))`;
      }
    }, { passive: true });
  },

  // =============================================
  // Data Fetching with fallback
  // =============================================
  
  async fetchData(url, fallback) {
    try {
      const response = await fetch(url);
      if (response.ok) return await response.json();
      throw new Error('API unavailable');
    } catch (err) {
      console.log(`API unavailable: ${url}, using fallback data`);
      return typeof fallback === 'function' ? fallback() : fallback;
    }
  },

  // =============================================
  // Formatting Utilities
  // =============================================
  
  formatHashrate(h) {
    if (!h || h === 0) return '0 H/s';
    if (h >= 1e15) return (h/1e15).toFixed(2) + ' PH/s';
    if (h >= 1e12) return (h/1e12).toFixed(2) + ' TH/s';
    if (h >= 1e9) return (h/1e9).toFixed(2) + ' GH/s';
    if (h >= 1e6) return (h/1e6).toFixed(2) + ' MH/s';
    if (h >= 1e3) return (h/1e3).toFixed(2) + ' KH/s';
    return h.toFixed(0) + ' H/s';
  },

  formatNumber(n) {
    if (!n || n === 0) return '0';
    if (n >= 1e9) return (n/1e9).toFixed(2) + 'B';
    if (n >= 1e6) return (n/1e6).toFixed(2) + 'M';
    if (n >= 1e3) return (n/1e3).toFixed(1) + 'K';
    return n.toLocaleString();
  },

  formatBTC(n) {
    if (!n || n === 0) return '0.00';
    return n.toFixed(8).replace(/\.?0+$/, '');
  },

  formatPercent(n) {
    if (n === undefined || n === null) return '0%';
    return (n * 100).toFixed(1) + '%';
  },

  formatTime(seconds) {
    if (seconds < 60) return seconds + 's';
    if (seconds < 3600) return Math.floor(seconds/60) + 'm ' + (seconds % 60) + 's';
    if (seconds < 86400) return Math.floor(seconds/3600) + 'h ' + Math.floor((seconds%3600)/60) + 'm';
    return Math.floor(seconds/86400) + 'd ' + Math.floor((seconds%86400)/3600) + 'h';
  },

  formatDate(timestamp) {
    if (!timestamp) return 'N/A';
    const d = new Date(timestamp);
    return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
  },

  formatDateTime(timestamp) {
    if (!timestamp) return 'N/A';
    const d = new Date(timestamp);
    return d.toLocaleString('en-US', { 
      month: 'short', day: 'numeric', year: 'numeric',
      hour: '2-digit', minute: '2-digit'
    });
  },

  formatDistanceToNow(timestamp) {
    if (!timestamp) return 'N/A';
    const diff = Math.floor((Date.now() - new Date(timestamp).getTime()) / 1000);
    return this.formatTime(Math.abs(diff)) + ' ago';
  },

  truncateHash(hash, len = 8) {
    if (!hash) return 'N/A';
    if (hash.length <= len * 2 + 3) return hash;
    return hash.substring(0, len) + '...' + hash.substring(hash.length - len);
  },

  // =============================================
  // Status helpers
  // =============================================

  statusBadge(status) {
    const map = {
      'online': '<span class="badge badge-green">🟢 Online</span>',
      'offline': '<span class="badge badge-red">🔴 Offline</span>',
      'syncing': '<span class="badge badge-orange">🟡 Syncing</span>',
      'active': '<span class="badge badge-green">✅ Active</span>',
      'pending': '<span class="badge badge-orange">⏳ Pending</span>',
      'verified': '<span class="badge badge-green">✅ Verified</span>',
      'passed': '<span class="badge badge-green">✅ PASS</span>',
      'failed': '<span class="badge badge-red">❌ FAIL</span>',
    };
    return map[status] || `<span class="badge badge-blue">${status}</span>`;
  },

  statusDot(status) {
    const map = {
      'up': 'status-dot-green',
      'down': 'status-dot-red',
      'degraded': 'status-dot-yellow',
    };
    return `<span class="status-dot ${map[status] || 'status-dot-green'}"></span>`;
  },

  // =============================================
  // Form handling
  // =============================================

  getFormData(formElement) {
    const data = {};
    const formData = new FormData(formElement);
    for (const [key, value] of formData.entries()) {
      if (data[key]) {
        if (!Array.isArray(data[key])) data[key] = [data[key]];
        data[key].push(value);
      } else {
        data[key] = value;
      }
    }
    return data;
  },

  showMessage(element, message, type = 'success') {
    const icons = { success: '✅', error: '❌', info: 'ℹ️', warning: '⚠️' };
    element.innerHTML = `
      <div class="alert alert-${type}">
        <span>${icons[type] || 'ℹ️'}</span>
        <span>${message}</span>
      </div>
    `;
    element.style.display = 'block';
  },

  // =============================================
  // Local Storage
  // =============================================

  storage: {
    get(key, defaultValue = null) {
      try {
        const item = localStorage.getItem('udaya_' + key);
        return item ? JSON.parse(item) : defaultValue;
      } catch { return defaultValue; }
    },
    set(key, value) {
      try {
        localStorage.setItem('udaya_' + key, JSON.stringify(value));
      } catch { /* ignore */ }
    },
    remove(key) {
      try { localStorage.removeItem('udaya_' + key); } catch { /* ignore */ }
    }
  },

  // =============================================
  // Chart helpers
  // =============================================

  createSparkline(canvasId, data, color = '#F59E0B') {
    const canvas = document.getElementById(canvasId);
    if (!canvas || !data || data.length < 2) return;
    const ctx = canvas.getContext('2d');
    const w = canvas.width;
    const h = canvas.height;
    
    ctx.clearRect(0, 0, w, h);
    
    const min = Math.min(...data);
    const max = Math.max(...data);
    const range = max - min || 1;
    
    ctx.beginPath();
    ctx.strokeStyle = color;
    ctx.lineWidth = 2;
    ctx.lineJoin = 'round';
    
    data.forEach((val, i) => {
      const x = (i / (data.length - 1)) * w;
      const y = h - ((val - min) / range) * (h - 4) - 2;
      i === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y);
    });
    
    ctx.stroke();
    
    const lastX = w;
    const lastY = h - ((data[data.length - 1] - min) / range) * (h - 4) - 2;
    ctx.lineTo(lastX, h);
    ctx.lineTo(0, h);
    ctx.closePath();
    
    const gradient = ctx.createLinearGradient(0, 0, 0, h);
    gradient.addColorStop(0, color + '30');
    gradient.addColorStop(1, color + '05');
    ctx.fillStyle = gradient;
    ctx.fill();
  },

  createBarChart(canvasId, data, labels, colors) {
    const canvas = document.getElementById(canvasId);
    if (!canvas || !data || !data.length) return;
    const ctx = canvas.getContext('2d');
    const w = canvas.width;
    const h = canvas.height;
    const maxVal = Math.max(...data) || 1;
    const barCount = data.length;
    const gap = 4;
    const barWidth = Math.min((w - gap * (barCount + 1)) / barCount, 40);
    const offsetX = (w - (barWidth * barCount + gap * (barCount - 1))) / 2;
    
    ctx.clearRect(0, 0, w, h);
    
    data.forEach((val, i) => {
      const barH = (val / maxVal) * (h - 10);
      const x = offsetX + i * (barWidth + gap);
      const y = h - 5 - barH;
      
      ctx.fillStyle = colors ? colors[i % colors.length] : '#F59E0B';
      ctx.beginPath();
      ctx.roundRect(x, y, barWidth, barH, 3);
      ctx.fill();
    });
  },

  // =============================================
  // Copy to clipboard
  // =============================================

  copyToClipboard(text) {
    if (navigator.clipboard) {
      navigator.clipboard.writeText(text).catch(() => {});
    }
  },

  // =============================================
  // Keyboard Navigation
  // =============================================

  initKeyboardSupport() {
    document.addEventListener('keydown', (e) => {
      if (e.key === 'Tab') {
        const nav = document.querySelector('.nav');
        const navLinks = document.querySelector('.nav-links');
        if (navLinks?.classList.contains('open')) {
          const focusable = navLinks.querySelectorAll('a');
          const first = focusable[0];
          const last = focusable[focusable.length - 1];
          
          if (e.shiftKey && document.activeElement === first) {
            e.preventDefault();
            last.focus();
          } else if (!e.shiftKey && document.activeElement === last) {
            e.preventDefault();
            first.focus();
          }
        }
      }
    });
  },

  // =============================================
  // Init
  // =============================================

  init() {
    console.log(`Udaya Ecosystem v${this.version} initialized`);
    
    // Initialize all components
    this.initParticles();
    this.initThreeGlobe();
    this.initNav();
    this.initRevealAnimations();
    this.initCounters();
    this.initSparklines();
    this.initKeyboardSupport();
    this.initNetworkViz();
    this.initScrollProgress();
    this.initLiveMetrics();
    this.initParallax();
    
    // Expose Udaya globally for other scripts
    window.Udaya = this;
  }
};

// =============================================
// Auto-initialize on DOM ready
// =============================================
document.addEventListener('DOMContentLoaded', () => Udaya.init());