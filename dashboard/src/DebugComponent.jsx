import React, { useEffect, useState } from 'react';

const DebugComponent = () => {
  const [debugInfo, setDebugInfo] = useState({
    nodeUrl: '',
    apiStatus: 'checking...',
    statsData: null,
    error: null,
  });

  useEffect(() => {
    const test = async () => {
      // Use same logic as TrinityChainDashboard
      const getNodeUrl = () => {
    // Local development
    if (window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1') {
      return 'http://localhost:3000';
    }
    // GitHub Codespaces: replace -5173. with -3000. in the hostname
    if (window.location.hostname.includes('.github.dev')) {
      return window.location.origin.replace('-5173.', '-3000.');
    }
    // Production: try to use API on same domain
    if (window.location.hostname.includes('render.com') || window.location.hostname.includes('vercel.app')) {
      return window.location.origin;
    }
    // Fallback
    return `${window.location.protocol}//${window.location.hostname}:3000`;
  };
      
  const nodeUrl = getNodeUrl();
      
      console.log('🔍 Debug Component Initialized');
      console.log('Node URL:', nodeUrl);
      console.log('Window location:', window.location.href);
      
      setDebugInfo(prev => ({
        ...prev,
        nodeUrl: nodeUrl || 'Using relative URL'
      }));

      try {
        console.log('Fetching from:', `${nodeUrl}/api/blockchain/stats`);
        const response = await fetch(`${nodeUrl}/api/blockchain/stats`, {
          credentials: 'include', // Required for Codespaces forwarded ports
        });
        console.log('Response status:', response.status);
        console.log('Response headers:', {
          contentType: response.headers.get('content-type'),
          corsOrigin: response.headers.get('access-control-allow-origin'),
        });

        if (!response.ok) {
          throw new Error(`HTTP ${response.status}`);
        }

        const data = await response.json();
        console.log('✓ Stats loaded:', data);
        
        setDebugInfo(prev => ({
          ...prev,
          apiStatus: 'Connected ✓',
          statsData: data,
        }));
      } catch (error) {
        console.error('✗ Error:', error);
        setDebugInfo(prev => ({
          ...prev,
          apiStatus: 'Failed ✗',
          error: error.message,
        }));
      }
    };

    test();
  }, []);

  return (
    <div style={{
      padding: '20px',
      backgroundColor: '#1e293b',
      color: '#e2e8f0',
      borderRadius: '8px',
      fontFamily: 'monospace',
      fontSize: '12px',
      maxHeight: '400px',
      overflow: 'auto',
    }}>
      <h3>🔍 Dashboard Debug Info</h3>
      <p><strong>Node URL:</strong> {debugInfo.nodeUrl}</p>
      <p><strong>API Status:</strong> {debugInfo.apiStatus}</p>
      
      {debugInfo.error && (
        <p style={{ color: '#ff4444' }}>
          <strong>Error:</strong> {debugInfo.error}
        </p>
      )}
      
      {debugInfo.statsData && (
        <pre style={{ backgroundColor: '#0f172a', padding: '10px' }}>
          {JSON.stringify(debugInfo.statsData, null, 2)}
        </pre>
      )}
      
      <details>
        <summary>Open Browser Console (F12) for detailed logs</summary>
        <p>Check the Network tab to see API requests</p>
      </details>
    </div>
  );
};

export default DebugComponent;
