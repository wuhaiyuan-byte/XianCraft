import { useState, useEffect, useRef, useCallback } from 'react';
import Convert from 'ansi-to-html';
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

// Matches the Rust `ServerMessage` enum
type ServerMessage =
  | { type: 'Description', payload: string }
  | { type: 'Info', payload: string }
  | { type: 'Error', payload: string }
  | { type: 'FullState', state: PlayerState } // For initial load
  | { type: 'DerivedStatsUpdate', state: DerivedStats }; // For frequent updates

// Matches the Rust `ClientMessage` enum
type ClientMessage =
  | { type: 'Login', user_id: string }
  | { type: 'Command', command: string };

//---------- React Component ----------//

const convert = new Convert({
  newline: true,
  escapeXML: true,
});

function App() {
  // State Management
  const [messages, setMessages] = useState<string[]>(['--- Please enter your name to begin ---']);
  const [input, setInput] = useState('');
  const [isLoggedIn, setIsLoggedIn] = useState(false);
  const [username, setUsername] = useState('');
  const [shouldConnect, setShouldConnect] = useState(false);
  
  const [fullPlayerState, setFullPlayerState] = useState<PlayerState | null>(null);
  const [derivedStats, setDerivedStats] = useState<DerivedStats | null>(null);

  // Refs for precise element control
  const ws = useRef<WebSocket | null>(null);
  const messagesEndRef = useRef<null | HTMLDivElement>(null);
  const usernameRef = useRef('');

  // Sync ref with state for the websocket closure
  useEffect(() => {
    usernameRef.current = username;
  }, [username]);

  // Effect to lock screen height and prevent layout jumps on mobile when keyboard appears.
  useEffect(() => {
    const setVh = () => {
      const vh = window.innerHeight * 0.01;
      document.documentElement.style.setProperty('--vh', `${vh}px`);
    };

    setVh(); // Set it once on initial load
  }, []);
  
  // Scroll to bottom when new messages arrive
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  // WebSocket connection logic
  useEffect(() => {
    if (!shouldConnect) return;

    const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${wsProtocol}//${window.location.host}/ws`;
    const socket = new WebSocket(wsUrl);
    ws.current = socket;
    let heartbeatInterval: number | null = null;

    socket.onopen = () => {
      console.log('Connection established.');
      socket.send(JSON.stringify({ type: 'Login', user_id: usernameRef.current }));
      setIsLoggedIn(true);

      // Heartbeat to keep connection alive
      heartbeatInterval = window.setInterval(() => {
        if (socket.readyState === WebSocket.OPEN) {
          socket.send(JSON.stringify({ type: 'Command', command: 'heartbeat' }));
        }
      }, 30000);
    };

    socket.onmessage = (event) => {
      const serverMessage: ServerMessage = JSON.parse(event.data);

      switch (serverMessage.type) {
        case 'Description':
        case 'Info': {
          const html = convert.toHtml(serverMessage.payload);
          setMessages(prev => [...prev, html]);
          break;
        }
        case 'Error': {
          const html = convert.toHtml(serverMessage.payload);
          setMessages(prev => [...prev, `<span class="text-red-500">${html}</span>`]);
          break;
        }
        case 'FullState':
          setFullPlayerState(serverMessage.state);
          setDerivedStats(serverMessage.state.derived);
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
      if (heartbeatInterval) clearInterval(heartbeatInterval);
      setMessages(prev => [...prev, '--- Disconnected. Please refresh to reconnect. ---']);
      setIsLoggedIn(false);
      setShouldConnect(false);
      ws.current = null;
    };

    socket.onerror = (error) => {
      console.error('WebSocket error:', error);
      if (heartbeatInterval) clearInterval(heartbeatInterval);
      setMessages(prev => [...prev, '--- Connection error. Is the server running? ---']);
      setIsLoggedIn(false);
      setShouldConnect(false);
    };

    return () => {
      if (heartbeatInterval) clearInterval(heartbeatInterval);
      if (socket.readyState === WebSocket.OPEN) socket.close();
    };
  }, [shouldConnect]);

  // Action Handlers
  const sendMessage = useCallback((message: ClientMessage) => {
    if (ws.current && ws.current.readyState === WebSocket.OPEN) {
      ws.current.send(JSON.stringify(message));
    } else {
      setMessages(prev => [...prev, '<span class="text-red-500">--- Error: Not connected. ---</span>']);
    }
  }, []);

  const handleLogin = () => {
    if (username.trim()) setShouldConnect(true);
    else setMessages(['--- Please enter a name. ---']);
  };
  
  // A unified command handler that prevents focus stealing
  const handleCommandAction = useCallback((command: string) => {
      if (command.trim()) {
          sendMessage({ type: 'Command', command });
      }
  }, [sendMessage]);
  
  const handleSendCommand = () => {
    if (input.trim()) {
      handleCommandAction(input);
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
    <div className="whitespace-pre-wrap font-mono">
      {messages.map((msg, index) => (
        <div key={index} dangerouslySetInnerHTML={{ __html: msg }} />
      ))}
      <div ref={messagesEndRef} />
    </div>
  );

  const renderStatusPanel = () => (
    <div className="hidden md:flex w-full md:w-1/4 border-t-2 md:border-t-0 md:border-l border-gray-600 flex-col">
        <div className="p-3 md:p-4 border-b border-gray-600">
            <h2 className="text-base md:text-lg font-bold mb-2">Status</h2>
            {derivedStats ? (
                <div className="grid grid-cols-2 gap-x-4">
                    <span>HP:</span>      <span>{derivedStats.hp}/{derivedStats.max_hp}</span>
                    <span>MP:</span>      <span>{derivedStats.mp}/{derivedStats.max_mp}</span>
                    <span>Stamina:</span> <span>{derivedStats.stamina}/{derivedStats.max_stamina}</span>
                </div>
            ) : <p>Loading...</p>}
        </div>
        <div className="p-3 md:p-4 border-b border-gray-600 flex-grow overflow-y-auto">
            <h2 className="text-base md:text-lg font-bold mb-2">Attributes</h2>
            {fullPlayerState ? (
                <div className="grid grid-cols-2 gap-x-4">
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
    </div>
  );

  const renderMobileQuickActions = () => (
      <div className="md:hidden p-2 bg-gray-800 border-t border-gray-600">
        <div className="grid grid-cols-4 gap-1">
            <button onMouseDown={(e) => e.preventDefault()} onClick={() => handleCommandAction('look')} className="bg-gray-700 hover:bg-gray-600 text-white text-xs font-bold py-2 px-1 rounded">Look</button>
            <button onMouseDown={(e) => e.preventDefault()} onClick={() => handleCommandAction('go n')} className="bg-gray-700 hover:bg-gray-600 text-white text-xs font-bold py-2 px-1 rounded">N</button>
            <button onMouseDown={(e) => e.preventDefault()} onClick={() => handleCommandAction('status')} className="bg-gray-700 hover:bg-gray-600 text-white text-xs font-bold py-2 px-1 rounded">Status</button>
            <button onMouseDown={(e) => e.preventDefault()} onClick={() => handleCommandAction('attr')} className="bg-gray-700 hover:bg-gray-600 text-white text-xs font-bold py-2 px-1 rounded">Attr</button>
            
            <button onMouseDown={(e) => e.preventDefault()} onClick={() => handleCommandAction('go w')} className="bg-gray-700 hover:bg-gray-600 text-white text-xs font-bold py-2 px-1 rounded">W</button>
            <button onMouseDown={(e) => e.preventDefault()} onClick={() => handleCommandAction('go s')} className="bg-gray-700 hover:bg-gray-600 text-white text-xs font-bold py-2 px-1 rounded">S</button>
            <button onMouseDown={(e) => e.preventDefault()} onClick={() => handleCommandAction('go e')} className="bg-gray-700 hover:bg-gray-600 text-white text-xs font-bold py-2 px-1 rounded">E</button>
            <button onMouseDown={(e) => e.preventDefault()} onClick={() => handleCommandAction('help')} className="bg-gray-700 hover:bg-gray-600 text-white text-xs font-bold py-2 px-1 rounded">Help</button>
        </div>
    </div>
  );

  if (!isLoggedIn) {
    return (
      <div className="bg-gray-900 text-white h-screen-real flex flex-col justify-center items-center font-mono p-4 overflow-hidden">
        <div className="text-center w-full max-w-lg">
          <h1 className="text-xl md:text-2xl font-bold mb-4">Rust MUD</h1>
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
            <button onMouseDown={(e) => e.preventDefault()} onClick={handleLogin} className="bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded-r">Login</button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="bg-gray-900 text-white h-screen-real flex flex-col font-mono text-sm md:text-base overflow-hidden">
      <header className="bg-gray-800 p-3 md:p-4 border-b border-gray-600 flex justify-between items-center">
        <h1 className="text-lg md:text-xl font-bold">Rust MUD</h1>
        <div className='text-right text-xs md:text-sm'><p>Logged in as: <span className='font-bold'>{username}</span></p></div>
      </header>
      <div className="flex flex-col md:flex-row flex-grow overflow-hidden min-h-0">
        <div className="flex flex-col flex-grow w-full md:w-3/4 min-h-0">
          <main className="flex-grow p-4 overflow-y-auto">
            {renderMessages()}
          </main>
          {renderMobileQuickActions()}
          <footer className="bg-gray-800 p-2 md:p-4 border-t border-gray-600 flex-shrink-0 flex">
            <input
              type="text"
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyPress={handleKeyPress}
              className="bg-gray-700 text-white flex-grow rounded-l px-2 py-2 focus:outline-none"
              placeholder="Enter command..."
            />
            <button onMouseDown={(e) => e.preventDefault()} onClick={handleSendCommand} className="bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-3 md:px-4 rounded-r">Send</button>
          </footer>
        </div>
        {renderStatusPanel()}
      </div>
    </div>
  );
}

export default App;