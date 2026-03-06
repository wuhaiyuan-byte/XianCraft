import { useState, useEffect, useRef } from 'react';

function App() {
  const [messages, setMessages] = useState<string[]>([]);
  const [input, setInput] = useState('');
  const ws = useRef<WebSocket | null>(null);

  useEffect(() => {
    // Construct the WebSocket URL based on the page's location
    // This will connect to the Vite proxy server, which forwards it to Rust
    const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'; // FIX: Corrected 'https:s' to 'https:'
    const wsUrl = `${wsProtocol}//${window.location.host}/ws`;

    ws.current = new WebSocket(wsUrl);

    ws.current.onopen = () => {
      console.log('Connection established through proxy.');
      setMessages(prev => [...prev, '--- Connected to server ---']);
    };

    ws.current.onmessage = (event) => {
      const formattedMessage = event.data.replace(/\n/g, '\n');
      setMessages(prev => [...prev, formattedMessage]);
    };

    ws.current.onclose = () => {
      console.log('Disconnected from MUD server');
      setMessages(prev => [...prev, '--- Disconnected from server ---']);
    };
    
    ws.current.onerror = (error) => {
        console.error('WebSocket Error:', error);
        setMessages(prev => [...prev, '--- WebSocket Error ---']);
    };

    // Cleanup on component unmount
    return () => {
      ws.current?.close();
    };
  }, []);

  const handleSend = () => {
    if (input.trim() && ws.current?.readyState === WebSocket.OPEN) {
      ws.current.send(input);
      setMessages(prev => [...prev, `> ${input}`]);
      setInput('');
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') {
      handleSend();
    }
  };

  return (
    <div className="bg-gray-900 text-white min-h-screen flex flex-col font-mono">
      <header className="bg-gray-800 p-4 border-b border-gray-600">
        <h1 className="text-xl font-bold">Rust MUD Client</h1>
      </header>
      
      <main className="flex-grow p-4 overflow-y-auto">
        <pre className="whitespace-pre-wrap">
          {messages.join('\n')}
        </pre>
      </main>

      <footer className="bg-gray-800 p-4 border-t border-gray-600 flex">
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyPress={handleKeyPress}
          className="bg-gray-700 text-white flex-grow rounded-l px-2 py-1 focus:outline-none"
          placeholder="Enter command..."
        />
        <button 
          onClick={handleSend}
          className="bg-blue-600 hover:bg-blue-700 text-white font-bold py-1 px-4 rounded-r"
        >
          Send
        </button>
      </footer>
    </div>
  );
}

export default App;
