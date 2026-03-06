import { useState, useEffect, useRef, useCallback } from 'react';
import './index.css'; // Make sure our styles are imported

//---------- Type Definitions ----------//

// Matches the `BaseAttributes` struct in Rust
interface BaseAttributes {
  strength: number;
  agility: number;
  constitution: number;
  comprehension: number;
}

// Matches the `DerivedStats` struct in Rust
interface DerivedStats {
  hp: number;
  max_hp: number;
  mp: number;
  max_mp: number;
  stamina: number;
  max_stamina: number;
}

// Matches the `Progression` struct in Rust
interface Progression {
  level: number;
  experience: number;
  potential: number;
}

// This is the full player state, received once on login.
interface PlayerState {
  base: BaseAttributes;
  derived: DerivedStats;
  progression: Progression;
}

// Updated ServerMessage to handle both full state and partial updates
type ServerMessage =
  | { type: 'GameMessage', content: string }
  | { type: 'FullState', state: PlayerState } // For initial load
  | { type: 'DerivedStatsUpdate', state: DerivedStats }; // For frequent updates

type ClientMessage =
  | { type: 'Login', username: string }
  | { type: 'Command', command: string };

//---------- React Component ----------//

function App() {
  // State Management
  const [messages, setMessages] = useState<string[]>(['--- Please enter your name to begin ---']);
  const [input, setInput] = useState('');
  const [isLoggedIn, setIsLoggedIn] = useState(false);
  const [username, setUsername] = useState('');
  const [shouldConnect, setShouldConnect] = useState(false);
  
  const [fullPlayerState, setFullPlayerState] = useState<PlayerState | null>(null);
  const [derivedStats, setDerivedStats] = useState<DerivedStats | null>(null);

  const ws = useRef<WebSocket | null>(null);
  const messagesEndRef = useRef<null | HTMLDivElement>(null);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  useEffect(() => {
    if (!shouldConnect) return;

    const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${wsProtocol}//${window.location.host}/ws`;
    const socket = new WebSocket(wsUrl);
    ws.current = socket;

    socket.onopen = () => {
      console.log('Connection established.');
      socket.send(JSON.stringify({ type: 'Login', username }));
      setIsLoggedIn(true);
      setMessages(['--- Welcome! Connecting to the realm... ---']);
    };

    socket.onmessage = (event) => {
      const serverMessage: ServerMessage = JSON.parse(event.data);

      switch (serverMessage.type) {
        case 'GameMessage':
          // Replace newline characters with <br> tags for HTML rendering
          const formattedContent = serverMessage.content.replace(/\n/g, '<br />');
          setMessages(prev => [...prev, formattedContent]);
          break;
        case 'FullState':
          setFullPlayerState(serverMessage.state);
          setDerivedStats(serverMessage.state.derived); // Correctly sync derived stats on initial load
          break;
        case 'DerivedStatsUpdate':
          setDerivedStats(serverMessage.state);
          break;
        default:
          console.warn("Received unknown message type:", serverMessage);
      }
    };

    socket.onclose = () => {
      console.log('Connection closed.');
      setMessages(prev => [...prev, '--- Disconnected. Please refresh to reconnect. ---']);
      setIsLoggedIn(false);
      setShouldConnect(false);
      ws.current = null;
    };

    socket.onerror = (error) => {
      console.error('WebSocket error:', error);
      setMessages(prev => [...prev, '--- Connection error. Is the server running? ---']);
      setIsLoggedIn(false);
      setShouldConnect(false);
    };

    return () => {
      if (socket.readyState === WebSocket.OPEN) socket.close();
    };
  }, [shouldConnect, username]);

  const sendMessage = useCallback((message: ClientMessage) => {
    if (ws.current && ws.current.readyState === WebSocket.OPEN) {
      ws.current.send(JSON.stringify(message));
    } else {
      setMessages(prev => [...prev, '--- Error: Not connected. ---']);
    }
  }, []);

  const handleLogin = () => {
    if (username.trim()) setShouldConnect(true);
    else setMessages(['--- Please enter a name. ---']);
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

  const renderMessages = () => (
    <div className="whitespace-pre-wrap">
      {messages.map((msg, index) => (
        <div key={index} dangerouslySetInnerHTML={{ __html: msg }} />
      ))}
      <div ref={messagesEndRef} />
    </div>
  );

  const renderStatusPanel = () => (
    <div className="w-1/4 border-l border-gray-600 flex flex-col">
        <div className="p-4 border-b border-gray-600">
            <h2 className="text-lg font-bold mb-2">Status</h2>
            {derivedStats ? (
                <div className="grid grid-cols-2 gap-x-4 text-sm">
                    <span>HP:</span>      <span>{derivedStats.hp}/{derivedStats.max_hp}</span>
                    <span>MP:</span>      <span>{derivedStats.mp}/{derivedStats.max_mp}</span>
                    <span>Stamina:</span> <span>{derivedStats.stamina}/{derivedStats.max_stamina}</span>
                </div>
            ) : <p>Loading...</p>}
        </div>
        <div className="p-4 border-b border-gray-600 flex-grow overflow-y-auto">
            <h2 className="text-lg font-bold mb-2">Attributes</h2>
            {fullPlayerState ? (
                <div className="grid grid-cols-2 gap-x-4 text-sm">
                    <span>Level:</span>     <span>{fullPlayerState.progression.level}</span>
                    <span>Experience:</span><span>{fullPlayerState.progression.experience}</span>
                    <span>Potential:</span> <span>{fullPlayerState.progression.potential}</span>
                    <span className="col-span-2 my-2 border-t border-gray-700"></span>
                    <span>Strength:</span>  <span>{fullPlayerState.base.strength}</span>
                    <span>Agility:</span>   <span>{fullPlayerState.base.agility}</span>
                    <span>Constitution:</span><span>{fullPlayerState.base.constitution}</span>
                    <span>Comprehension:</span><span>{fullPlayerState.base.comprehension}</span>
                </div>
            ) : <p>Loading...</p>}
        </div>
         <div className="p-4 border-t border-gray-600">
            <h2 className="text-lg font-bold mb-2">Quick Actions</h2>
            <div className="grid grid-cols-2 gap-2">
              <button onClick={() => sendMessage({ type: 'Command', command: 'look' })} className="bg-gray-700 hover:bg-gray-600 text-white font-bold py-2 px-4 rounded">Look</button>
              <button onClick={() => sendMessage({ type: 'Command', command: 'status' })} className="bg-gray-700 hover:bg-gray-600 text-white font-bold py-2 px-4 rounded">Status</button>
              <button onClick={() => sendMessage({ type: 'Command', command: 'go north' })} className="bg-gray-700 hover:bg-gray-600 text-white font-bold py-2 px-4 rounded">North</button>
              <button onClick={() => sendMessage({ type: 'Command', command: 'go south' })} className="bg-gray-700 hover:bg-gray-600 text-white font-bold py-2 px-4 rounded">South</button>
              <button onClick={() => sendMessage({ type: 'Command', command: 'go east' })} className="bg-gray-700 hover:bg-gray-600 text-white font-bold py-2 px-4 rounded">East</button>
              <button onClick={() => sendMessage({ type: 'Command', command: 'go west' })} className="bg-gray-700 hover:bg-gray-600 text-white font-bold py-2 px-4 rounded">West</button>
              <button onClick={() => sendMessage({ type: 'Command', command: 'help' })} className="bg-gray-700 hover:bg-gray-600 text-white font-bold py-2 px-4 rounded col-span-2">Help</button>
            </div>
        </div>
    </div>
  );

  if (!isLoggedIn) {
    return (
      <div className="bg-gray-900 text-white h-screen flex flex-col justify-center items-center font-mono">
        <div className="text-center w-full max-w-lg p-4">
          <h1 className="text-2xl font-bold mb-4">Rust MUD</h1>
          <div className='w-full bg-black border border-gray-600 p-2 mb-4 h-48 overflow-y-auto text-left'>
             {renderMessages()}
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
            <button onClick={handleLogin} className="bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded-r">Login</button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="bg-gray-900 text-white h-screen flex flex-col font-mono">
      <header className="bg-gray-800 p-4 border-b border-gray-600 flex justify-between items-center">
        <h1 className="text-xl font-bold">Rust MUD</h1>
        <div className='text-right'><p>Logged in as: <span className='font-bold'>{username}</span></p></div>
      </header>
      <div className="flex flex-grow overflow-hidden">
        <div className="flex flex-col flex-grow w-3/4">
          <main className="flex-grow p-4 overflow-y-auto">
            {renderMessages()}
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
            <button onClick={handleSendCommand} className="bg-blue-600 hover:bg-blue-700 text-white font-bold py-1 px-4 rounded-r">Send</button>
          </footer>
        </div>
        {renderStatusPanel()}
      </div>
    </div>
  );
}

export default App;
