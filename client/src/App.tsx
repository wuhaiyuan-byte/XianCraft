import { useState, useEffect, useRef, useCallback } from 'react';

//---------- Type Definitions ----------//

interface PlayerState {
  hp: number;
  max_hp: number;
  mp: number;
  max_mp: number;
  stamina: number;
  max_stamina: number;
}

type ServerMessage =
  | { type: 'GameMessage', content: string }
  | { type: 'PlayerStateUpdate', state: PlayerState };

type ClientMessage =
  | { type: 'Login', username: string }
  | { type: 'Command', command: string };

//---------- React Component ----------//

function App() {
  // State Management
  const [messages, setMessages] = useState<string[]>(['--- Please enter your name to begin ---']);
  const [input, setInput] = useState('');
  const [playerState, setPlayerState] = useState<PlayerState | null>(null);
  const [isLoggedIn, setIsLoggedIn] = useState(false);
  const [username, setUsername] = useState('');
  const [shouldConnect, setShouldConnect] = useState(false);
  
  const ws = useRef<WebSocket | null>(null);
  const messagesEndRef = useRef<null | HTMLDivElement>(null);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  // This effect hook manages the entire WebSocket lifecycle.
  // It runs only when `shouldConnect` is set to true.
  useEffect(() => {
    if (!shouldConnect) {
      return;
    }

    const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${wsProtocol}//${window.location.host}/ws`;
    const socket = new WebSocket(wsUrl);
    ws.current = socket;

    socket.onopen = () => {
      console.log('Connection established.');
      // Once connected, send the login message
      socket.send(JSON.stringify({ type: 'Login', username }));
      setIsLoggedIn(true);
      setMessages(['--- Welcome! Connecting to the realm... ---']);
    };

    socket.onmessage = (event) => {
      let serverMessage: ServerMessage;
      try {
        serverMessage = JSON.parse(event.data);
      } catch (error) {
        console.error("Failed to parse server message:", event.data);
        setMessages(prev => [...prev, `[RAW] ${event.data}`]);
        return;
      }

      switch (serverMessage.type) {
        case 'GameMessage':
          setMessages(prev => [...prev, serverMessage.content]);
          break;
        case 'PlayerStateUpdate':
          setPlayerState(serverMessage.state);
          break;
        default:
          console.warn("Received unknown message type:", serverMessage);
      }
    };

    socket.onclose = () => {
      console.log('Connection closed.');
      setMessages(prev => [...prev, '--- Disconnected from server. Please refresh to reconnect. ---']);
      setIsLoggedIn(false);
      setPlayerState(null);
      setShouldConnect(false); // Allow reconnecting
      ws.current = null;
    };

    socket.onerror = (error) => {
      console.error('WebSocket error:', error);
      setMessages(prev => [...prev, '--- Connection error. Is the server running? ---']);
      setIsLoggedIn(false);
      setShouldConnect(false);
    };

    // Cleanup function: this is called when the component unmounts or before the effect re-runs.
    return () => {
      if (socket.readyState === WebSocket.OPEN) {
        console.log("Closing WebSocket connection...");
        socket.close();
      }
    };
  }, [shouldConnect, username]); // Effect depends on these state variables

  // A stable function to send any client message
  const sendMessage = useCallback((message: ClientMessage) => {
    if (ws.current && ws.current.readyState === WebSocket.OPEN) {
      ws.current.send(JSON.stringify(message));
    } else {
      setMessages(prev => [...prev, '--- Error: Not connected to server. ---']);
    }
  }, []);

  // This function now just signals that a connection should be attempted.
  const handleLogin = () => {
    if (username.trim()) {
      setShouldConnect(true);
    } else {
      setMessages(['--- Please enter a name. ---']);
    }
  };
  
  const handleSendCommand = () => {
    if (input.trim()) {
      sendMessage({ type: 'Command', command: input });
      setInput('');
    }
  };

  const handleKeyPress = (event: React.KeyboardEvent<HTMLInputElement>) => {
    if (event.key === 'Enter') {
      isLoggedIn ? handleSendCommand() : handleLogin();
    }
  };

  //---------- JSX Rendering ----------//

  // Login View
  if (!isLoggedIn) {
    return (
      <div className="bg-gray-900 text-white h-screen flex flex-col justify-center items-center font-mono">
        <div className="text-center w-full max-w-lg p-4">
          <h1 className="text-2xl font-bold mb-4">Rust MUD</h1>
          <div className='w-full bg-black border border-gray-600 p-2 mb-4 h-48 overflow-y-auto text-left'>
            <pre className="whitespace-pre-wrap">{messages.join('\n')}</pre>
            <div ref={messagesEndRef} />
          </div>
          <div className="flex">
            <input
              type="text"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              onKeyPress={handleKeyPress}
              className="bg-gray-700 text-white flex-grow rounded-l px-4 py-2 focus:outline-none"
              placeholder="Enter your name"
              autoFocus
            />
            <button
              onClick={handleLogin}
              className="bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded-r"
            >
              Login
            </button>
          </div>
        </div>
      </div>
    );
  }

  // Main Game View
  return (
    <div className="bg-gray-900 text-white h-screen flex flex-col font-mono">
      <header className="bg-gray-800 p-4 border-b border-gray-600 flex justify-between items-center">
        <h1 className="text-xl font-bold">Rust MUD</h1>
        <div className='text-right'>
          <p>Logged in as: <span className='font-bold'>{username}</span></p>
        </div>
      </header>
      <div className="flex flex-grow overflow-hidden">
        <div className="flex flex-col flex-grow w-3/4">
          <main className="flex-grow p-4 overflow-y-auto">
            <pre className="whitespace-pre-wrap">{messages.join('\n')}</pre>
            <div ref={messagesEndRef} />
          </main>
          <footer className="bg-gray-800 p-4 border-t border-gray-600 flex-shrink-0 flex">
            <input
              type="text"
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyPress={handleKeyPress}
              className="bg-gray-700 text-white flex-grow rounded-l px-2 py-1 focus:outline-none"
              placeholder="Enter command..."
              autoFocus
            />
            <button onClick={handleSendCommand} className="bg-blue-600 hover:bg-blue-700 text-white font-bold py-1 px-4 rounded-r">
              Send
            </button>
          </footer>
        </div>
        <div className="w-1/4 border-l border-gray-600 flex flex-col">
          <div className="h-1/2 p-4 border-b border-gray-600 overflow-y-auto">
            <h2 className="text-lg font-bold mb-2">Player Status</h2>
            {playerState ? (
              <div>
                <p>Health: {playerState.hp}/{playerState.max_hp}</p>
                <p>Mana: {playerState.mp}/{playerState.max_mp}</p>
                <p>Stamina: {playerState.stamina}/{playerState.max_stamina}</p>
              </div>
            ) : (
              <p>Loading status...</p>
            )}
          </div>
          <div className="h-1/2 p-4 overflow-y-auto">
            <h2 className="text-lg font-bold mb-2">Quick Actions</h2>
            <div className="grid grid-cols-2 gap-2">
              <button onClick={() => sendMessage({ type: 'Command', command: 'look' })} className="bg-gray-700 hover:bg-gray-600 text-white font-bold py-2 px-4 rounded">Look</button>
              <button onClick={() => sendMessage({ type: 'Command', command: 'go north' })} className="bg-gray-700 hover:bg-gray-600 text-white font-bold py-2 px-4 rounded">North</button>
              <button onClick={() => sendMessage({ type: 'Command', command: 'go south' })} className="bg-gray-700 hover:bg-gray-600 text-white font-bold py-2 px-4 rounded">South</button>
              <button onClick={() => sendMessage({ type: 'Command', command: 'go east' })} className="bg-gray-700 hover:bg-gray-600 text-white font-bold py-2 px-4 rounded">East</button>
              <button onClick={() => sendMessage({ type: 'Command', command: 'go west' })} className="bg-gray-700 hover:bg-gray-600 text-white font-bold py-2 px-4 rounded">West</button>
              <button onClick={() => sendMessage({ type: 'Command', command: 'help' })} className="bg-gray-700 hover:bg-gray-600 text-white font-bold py-2 px-4 rounded">Help</button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;
