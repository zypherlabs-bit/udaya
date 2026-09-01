// =============================================
// Udaya Ecosystem API Server
// =============================================

const http = require('http');
const fs = require('fs');
const path = require('path');

const PORT = 3000;
const MIME_TYPES = {
    '.html': 'text/html; charset=utf-8',
    '.css': 'text/css; charset=utf-8',
    '.js': 'application/javascript; charset=utf-8',
    '.json': 'application/json; charset=utf-8',
    '.png': 'image/png',
    '.jpg': 'image/jpeg',
    '.svg': 'image/svg+xml',
    '.ico': 'image/x-icon',
};

// Live network simulation: values advance over time so the site feels alive
const SIM_START = Date.now();
const BASE_BLOCK_HEIGHT = 1245100;
const BLOCK_TIME_SECONDS = 60;

// Simple in-memory store for ecosystem data
const ecosystemStore = {
    stats: {
        blockHeight: BASE_BLOCK_HEIGHT,
        blockTime: BLOCK_TIME_SECONDS,
        hashrate: 4.42e15,          // ~4.42 PH/s
        activeNodes: 12,
        testnetNodes: 5,
        dailyTransactions: 4890,
        transactionsInBlock: 68,
        activeWallets: 8942,
        activeMiners: 125,
        pools: 1,
        developers: 96,
        merchants: 12,
        treasuryBalance: 4500000,
        governanceParticipation: 12,
        sdkDownloads: 350,
        explorerQueries: 1280,
        rpcRequests: 2468,
        mempoolSize: 142,
        nakamotoCoefficient: 5,
        giniCoefficient: 0.2,
        hhiScore: 1280,
        rewardPerBlock: 50,
        difficulty: 284172900,
    },
    verifications: [],
    bountyReports: [],
    nodeStatus: [
        { name: 'seed-us.Udaya.org', region: 'US-East', provider: 'AWS', status: 'up', height: 0, version: '1.0.0' },
        { name: 'seed-eu.Udaya.org', region: 'Europe', provider: 'Hetzner', status: 'up', height: 0, version: '1.0.0' },
        { name: 'seed-ap.Udaya.org', region: 'Asia-Pacific', provider: 'Linode', status: 'up', height: 0, version: '1.0.0' },
        { name: 'node-us-west', region: 'US-West', provider: 'DigitalOcean', status: 'up', height: 0, version: '1.0.0' },
        { name: 'node-global', region: 'Global', provider: 'Vultr', status: 'up', height: 0, version: '1.0.0' },
    ],
    community: {
        totalScore: 1450,
        verificationRate: 85,
        rewardsDistributed: 48,
        avgScore: 3.2,
        UDYARewarded: 2500,
        ambassadors: 45,
        contributors: 18,
        activeMembers: 92,
    },
    integrations: {
        sdkDownloads: 350,
        contributors: 12,
        pullRequests: 28,
        githubStars: 75,
    }
};

function liveStats() {
    const minutes = Math.floor((Date.now() - SIM_START) / 60000);
    const height = BASE_BLOCK_HEIGHT + Math.floor(minutes / (BLOCK_TIME_SECONDS / 60));
    const wallets = ecosystemStore.stats.activeWallets + Math.floor(minutes / 2);
    const dailyTx = ecosystemStore.stats.dailyTransactions + Math.floor(minutes / 3);
    const hashReturn = ecosystemStore.stats.hashrate * (1 + 0.03 * Math.sin(minutes / 90));
    return {
        ...ecosystemStore.stats,
        blockHeight: height,
        activeWallets: wallets,
        dailyTransactions: dailyTx,
        treasuryBalance: ecosystemStore.stats.treasuryBalance + Math.floor(minutes / 2) * 50,
        hashrate: hashReturn,
        mempoolSize: 90 + Math.floor(Math.abs(Math.sin(minutes / 45))) * 60 + (minutes % 30),
        difficulty: Math.round(ecosystemStore.stats.difficulty + Math.floor(minutes / 60) * 962),
        explorerQueries: ecosystemStore.stats.explorerQueries + minutes + Math.floor(Math.sin(minutes / 40) * 30),
        rpcRequests: ecosystemStore.stats.rpcRequests + minutes * 2 + Math.floor(Math.cos(minutes / 40) * 60),
        probes: minutes * 37,
        secondsToNextBlock: Math.max(0, Math.ceil(((SIM_START + (minutes + 1) * 60000) - Date.now()) / 1000)),
        updatedAt: new Date().toISOString(),
    };
}

// API Routes
const apiRoutes = {
    '/api/stats': () => liveStats(),
    '/api/nodes': () => ecosystemStore.nodeStatus,
    '/api/health': () => ({
        status: 'healthy',
        uptime: 99.9,
        rpcUptime: 99.9,
        nodeUptime: 99.8,
        incidents: 0,
        timestamp: new Date().toISOString()
    }),
    '/api/community': () => ecosystemStore.community,
    '/api/integrations': () => ecosystemStore.integrations,
    '/api/verification/count': () => ({
        verified: 128,
        passRate: 92,
        badgesIssued: 156,
        successfulSyncs: 45,
        walletRecoveries: 38,
        miningSessions: 22,
        txTests: 51
    }),
    '/api/ecosystem/summary': () => ({
        ...liveStats(),
        ...ecosystemStore.community,
        year1Targets: {
            wallets: { current: 8942, target: 10000 },
            nodes: { current: 5, target: 500 },
            miners: { current: 125, target: 100 },
            dailyTx: { current: 4890, target: 5000 },
            pools: { current: 1, target: 5 },
            developers: { current: 96, target: 100 },
            merchants: { current: 12, target: 50 },
            exchanges: { current: 3, target: 3 }
        },
        social: {
            totalFollowers: 5200,
            totalPosts: 500,
            avgEngagement: 15,
            platforms: {
                twitter: 2500,
                telegram: 1800,
                discord: 850,
                youtube: 320,
                linkedin: 450,
                github: 75
            }
        }
    }),
    '/api/metrics/history': () => ({
        wallets: [120, 210, 360, 540, 800, 1180, 1650, 2240, 3050, 3980, 4890, 5620, 6480, 7220, 7980, 8610, 9020, 9340, 9560, 9820, 9900, 10120, 10480, 10890, 11400, 11900, 12380, 12920, 13310, 13690],
        nodes: [1, 1, 2, 2, 3, 3, 3, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5],
        transactions: [412, 438, 451, 470, 496, 512, 530, 548, 575, 603, 640, 672, 705, 788, 812, 867, 940, 1005, 1080, 1135, 1210, 1280, 1355, 1420, 1490, 1580, 1650, 1730, 1815, 1890],
        developers: [8, 10, 14, 19, 24, 30, 36, 42, 48, 55, 60, 66, 71, 76, 80, 84, 86, 88, 90, 91, 92, 93, 93, 94, 94, 95, 95, 96, 96, 96]
    })
};

// POST routes
const apiPostRoutes = {
    '/api/register': (body) => {
        const id = 'BF-' + Date.now().toString(36).toUpperCase();
        ecosystemStore.verifications.push({ ...body, id, date: new Date().toISOString(), status: 'pending' });
        return { success: true, id, message: 'Registration submitted for verification' };
    },
    '/api/bounty/submit': (body) => {
        const id = 'BF-BUG-' + Date.now().toString(36).toUpperCase();
        ecosystemStore.bountyReports.push({ ...body, id, date: new Date().toISOString(), status: 'received' });
        return { success: true, id, message: 'Bug report submitted. Our security team will respond within 48 hours.' };
    },
    '/api/faucet/claim': (body) => {
        const txid = '0x' + Date.now().toString(16) + Math.random().toString(16).substring(2, 10);
        return { success: true, txid, amount: 10, message: '10 UDYA test tokens sent to ' + body.address };
    }
};

function parseBody(req) {
    return new Promise((resolve) => {
        let body = '';
        req.on('data', chunk => body += chunk);
        req.on('end', () => {
            try { resolve(JSON.parse(body)); }
            catch { resolve({}); }
        });
    });
}

const server = http.createServer(async (req, res) => {
    // CORS headers
    res.setHeader('Access-Control-Allow-Origin', '*');
    res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
    res.setHeader('Access-Control-Allow-Headers', 'Content-Type');

    if (req.method === 'OPTIONS') {
        res.writeHead(204);
        res.end();
        return;
    }

    const url = new URL(req.url, `http://${req.headers.host}`);
    const pathname = url.pathname;

    // API Routes
    if (pathname.startsWith('/api/')) {
        res.setHeader('Content-Type', 'application/json');

        if (req.method === 'GET') {
            const handler = apiRoutes[pathname];
            if (handler) {
                const data = handler();
                // Add cache headers for performance
                res.setHeader('Cache-Control', 'public, max-age=30');
                res.writeHead(200);
                res.end(JSON.stringify(data, null, 2));
            } else {
                res.writeHead(404);
                res.end(JSON.stringify({ error: 'API endpoint not found' }));
            }
            return;
        }

        if (req.method === 'POST') {
            const handler = apiPostRoutes[pathname];
            if (handler) {
                const body = await parseBody(req);
                const result = handler(body);
                res.writeHead(200);
                res.end(JSON.stringify(result, null, 2));
            } else {
                res.writeHead(404);
                res.end(JSON.stringify({ error: 'API endpoint not found' }));
            }
            return;
        }
    }

    // Serve centralized branding assets
    if (pathname.startsWith('/branding/')) {
        const brandDir = path.resolve(__dirname, '..', '..', 'assets', 'branding', 'logo');
        const fileName = pathname.substring('/branding/'.length);
        const filePath = path.join(brandDir, fileName);
        const ext = path.extname(filePath);
        const contentType = MIME_TYPES[ext] || 'application/octet-stream';
        try {
            const content = fs.readFileSync(filePath);
            res.writeHead(200, { 'Content-Type': contentType });
            res.end(content);
        } catch (err) {
            res.writeHead(404);
            res.end('Not Found');
        }
        return;
    }

    if (pathname.startsWith('/logo/')) {
        const logoDir = path.resolve(__dirname, '..', '..', 'assets', 'logo');
        const fileName = pathname.substring('/logo/'.length);
        const filePath = path.join(logoDir, fileName);
        const ext = path.extname(filePath);
        const contentType = MIME_TYPES[ext] || 'application/octet-stream';
        try {
            const content = fs.readFileSync(filePath);
            res.writeHead(200, { 'Content-Type': contentType });
            res.end(content);
        } catch (err) {
            res.writeHead(404);
            res.end('Not Found');
        }
        return;
    }

    // Static file serving (base is the website/ directory, one level up from js/)
    const baseDir = path.resolve(__dirname, '..');
    let filePath = path.join(baseDir, pathname === '/' ? 'index.html' : pathname);
    
    // Try .html extension if no extension
    if (!path.extname(filePath)) {
        const htmlPath = filePath + '.html';
        if (fs.existsSync(htmlPath)) {
            filePath = htmlPath;
        } else {
            filePath = path.join(filePath, 'index.html');
        }
    }

    const ext = path.extname(filePath);
    const contentType = MIME_TYPES[ext] || 'application/octet-stream';

    try {
        const content = fs.readFileSync(filePath);
        res.writeHead(200, { 'Content-Type': contentType });
        res.end(content);
    } catch (err) {
        // Fallback to index.html for SPA-like navigation
        try {
            const indexContent = fs.readFileSync(path.join(__dirname, 'index.html'));
            res.writeHead(200, { 'Content-Type': 'text/html; charset=utf-8' });
            res.end(indexContent);
        } catch {
            res.writeHead(404);
            res.end('Not Found');
        }
    }
});

server.listen(PORT, () => {
    console.log(`⚡ Udaya Ecosystem API Server running on http://localhost:${PORT}`);
    console.log(`📊 API endpoints:`);
    console.log(`   GET  /api/stats`);
    console.log(`   GET  /api/nodes`);
    console.log(`   GET  /api/health`);
    console.log(`   GET  /api/community`);
    console.log(`   GET  /api/integrations`);
    console.log(`   GET  /api/verification/count`);
    console.log(`   GET  /api/ecosystem/summary`);
    console.log(`   GET  /api/metrics/history`);
    console.log(`   POST /api/register`);
    console.log(`   POST /api/bounty/submit`);
    console.log(`   POST /api/faucet/claim`);
    console.log(`🌐 Serving static files from: ${__dirname}`);
});