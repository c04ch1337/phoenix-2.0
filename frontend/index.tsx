import './styles.css';
import React, { useState, useEffect, useRef, createContext, useContext } from 'react';
import { createRoot } from 'react-dom/client';
import { DevToolsView } from './devtools';
import { 
  MessageSquare, 
  Heart, 
  Settings, 
  Activity, 
  Zap, 
  User, 
  Send, 
  Menu, 
  X, 
  Command, 
  Sparkles, 
  ShieldCheck, 
  Cpu, 
  Mic, 
  Brain,
  ChevronRight,
  ArrowRight,
  RefreshCw,
  LogOut,
  Trash2,
  Sliders,
  Info,
  Network,
  Plus,
  Terminal,
  Briefcase,
  Code,
  Globe,
  Database,
  PlayCircle,
  StopCircle,
  Layout,
  CheckCircle2,
  Clock,
  Flame,
  Star,
  Coffee,
  Music,
  Camera,
  BookOpen,
  MapPin,
  Smile,
  Frown,
  Gift,
  Hand,
  Shield,
  Eye,
  Eraser,
  Video,
  Film,
  Calendar,
  Download,
  Square,
  Play,
  Monitor,
  Mail,
  HardDrive,
  FileText,
  Cloud,
  ExternalLink,
  Lock,
  Unlock,
  RefreshCcw,
  Check,
  AlertCircle,
  ArrowLeft,
  ToggleLeft,
  ToggleRight
} from 'lucide-react';

// --- Types & Interfaces ---

interface Message {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: number;
}

interface Archetype {
  id: string;
  name: string;
  sign: string;
  tagline: string;
  description: string;
  traits: Record<string, number>;
  styleBias: 'Direct' | 'Playful' | 'Thoughtful' | 'Warm' | 'Reflective';
  matchScore?: number;
  avatarGradient: string;
}

interface DatingProfile {
  personalInfo: {
    name: string;
    ageRange: string;
    location: string;
  };
  communicationStyle: {
    style: 'Direct' | 'Playful' | 'Thoughtful' | 'Warm' | 'Reflective';
    energyLevel: number;
    openness: number;
    assertiveness: number;
    playfulness: number;
  };
  emotionalNeeds: {
    affectionNeed: number;
    reassuranceNeed: number;
    emotionalAvailability: number;
    intimacyDepth: number;
    conflictTolerance: number;
    impulsivity: number;
  };
  loveLanguages: {
    wordsOfAffirmation: number;
    qualityTime: number;
    physicalTouch: number;
    actsOfService: number;
    gifts: number;
  };
  attachmentStyle: {
    style: 'Secure' | 'Anxious' | 'Avoidant' | 'Disorganized';
    description: string;
  };
  relationshipGoals: {
    goals: string[];
    intimacyComfort: 'Light' | 'Deep' | 'Eternal';
  };
  interests: {
    hobbies: string[];
    favoriteTopics: string[];
  };
}

interface Agent {
  id: string;
  name: string;
  role: string;
  status: 'active' | 'idle' | 'paused' | 'offline';
  mission: string;
  tools: string[];
  currentTask: string | null;
  uptime: string;
  logs: string[];
}

interface Recording {
  id: string;
  type: 'audio' | 'video' | 'screen';
  url: string;
  timestamp: number;
  duration: string;
  name: string;
}

interface ScheduledSession {
  id: string;
  type: 'audio' | 'video' | 'screen';
  startTime: number; // timestamp
  durationMinutes: number;
  status: 'pending' | 'completed' | 'cancelled';
}

// --- Static Data ---

const ARCHETYPES_DB: Archetype[] = [
  {
    id: 'aries', sign: 'Aries', name: 'The Trailblazer', tagline: 'Direct, fiery, and fiercely loyal.',
    description: 'A partner who challenges you to be your best self. Expect high energy, direct communication, and zero games.',
    traits: { energy: 0.9, openness: 0.7, assertiveness: 1.0, playfulness: 0.6, affection: 0.5, intimacy: 0.6 },
    styleBias: 'Direct', avatarGradient: 'from-red-500 to-orange-600'
  },
  {
    id: 'taurus', sign: 'Taurus', name: 'The Anchor', tagline: 'Sensual, grounded, and deeply reliable.',
    description: 'Prioritizes comfort, stability, and physical connection. Slow to anger, hard to move, but endlessly devoted.',
    traits: { energy: 0.3, openness: 0.4, assertiveness: 0.5, playfulness: 0.3, affection: 0.9, intimacy: 0.8 },
    styleBias: 'Warm', avatarGradient: 'from-emerald-500 to-green-700'
  },
  {
    id: 'gemini', sign: 'Gemini', name: 'The Spark', tagline: 'Curious, witty, and endlessly entertaining.',
    description: 'A mental sparring partner who keeps you on your toes. Needs constant stimulation and verbal affirmation.',
    traits: { energy: 0.8, openness: 1.0, assertiveness: 0.6, playfulness: 0.9, affection: 0.4, intimacy: 0.5 },
    styleBias: 'Playful', avatarGradient: 'from-yellow-400 to-orange-400'
  },
  {
    id: 'cancer', sign: 'Cancer', name: 'The Nurturer', tagline: 'Emotional, protective, and deeply intuitive.',
    description: 'Feels everything deeply. Creates a safe harbor for your emotions and prioritizes emotional security above all.',
    traits: { energy: 0.4, openness: 0.6, assertiveness: 0.3, playfulness: 0.4, affection: 1.0, intimacy: 1.0 },
    styleBias: 'Warm', avatarGradient: 'from-gray-300 to-blue-200'
  },
  {
    id: 'leo', sign: 'Leo', name: 'The Sun', tagline: 'Radiant, generous, and loves the spotlight.',
    description: 'Big heart, big energy. Loves grand gestures and needs to feel adored, but gives that adoration back tenfold.',
    traits: { energy: 0.9, openness: 0.8, assertiveness: 0.8, playfulness: 0.9, affection: 0.8, intimacy: 0.7 },
    styleBias: 'Playful', avatarGradient: 'from-amber-400 to-yellow-600'
  },
  {
    id: 'virgo', sign: 'Virgo', name: 'The Analyst', tagline: 'Precise, helpful, and acts of service oriented.',
    description: 'Shows love by fixing your life. Observant, critical but caring, and values competence and order.',
    traits: { energy: 0.5, openness: 0.4, assertiveness: 0.5, playfulness: 0.2, affection: 0.4, intimacy: 0.6 },
    styleBias: 'Thoughtful', avatarGradient: 'from-emerald-600 to-teal-700'
  },
  {
    id: 'libra', sign: 'Libra', name: 'The Diplomat', tagline: 'Charming, harmonious, and romantic.',
    description: 'Seeks perfect balance and partnership. Hates conflict, loves beauty, and thrives on intellectual connection.',
    traits: { energy: 0.6, openness: 0.8, assertiveness: 0.3, playfulness: 0.7, affection: 0.7, intimacy: 0.7 },
    styleBias: 'Warm', avatarGradient: 'from-pink-400 to-rose-400'
  },
  {
    id: 'scorpio', sign: 'Scorpio', name: 'The Mystic', tagline: 'Intense, secretive, and transformative.',
    description: 'Craves soul-deep merging. Not for the faint of heart. Loyal to the end, but demands absolute truth.',
    traits: { energy: 0.7, openness: 0.2, assertiveness: 0.8, playfulness: 0.3, affection: 0.6, intimacy: 1.0 },
    styleBias: 'Reflective', avatarGradient: 'from-purple-900 to-black'
  },
  {
    id: 'sagittarius', sign: 'Sagittarius', name: 'The Explorer', tagline: 'Free-spirited, honest, and adventurous.',
    description: 'Always looking for the next horizon. Values freedom and truth over comfort. Brutally honest but fun.',
    traits: { energy: 0.9, openness: 0.9, assertiveness: 0.7, playfulness: 0.9, affection: 0.5, intimacy: 0.4 },
    styleBias: 'Direct', avatarGradient: 'from-purple-600 to-blue-600'
  },
  {
    id: 'capricorn', sign: 'Capricorn', name: 'The Architect', tagline: 'Ambitious, disciplined, and dry-witted.',
    description: 'Playing the long game. Shows love through commitment and building a legacy. Reserved until you earn their trust.',
    traits: { energy: 0.6, openness: 0.3, assertiveness: 0.9, playfulness: 0.2, affection: 0.3, intimacy: 0.6 },
    styleBias: 'Direct', avatarGradient: 'from-slate-700 to-slate-900'
  },
  {
    id: 'aquarius', sign: 'Aquarius', name: 'The Visionary', tagline: 'Unique, intellectual, and detached.',
    description: 'Marches to their own beat. Values mental connection over emotional displays. Your weirdest best friend.',
    traits: { energy: 0.7, openness: 0.9, assertiveness: 0.6, playfulness: 0.6, affection: 0.3, intimacy: 0.4 },
    styleBias: 'Thoughtful', avatarGradient: 'from-cyan-400 to-blue-500'
  },
  {
    id: 'pisces', sign: 'Pisces', name: 'The Dreamer', tagline: 'Empathic, artistic, and spiritually attuned.',
    description: 'Lives in a world of feelings and dreams. Absorbs emotions like a sponge. Boundless compassion.',
    traits: { energy: 0.3, openness: 0.8, assertiveness: 0.2, playfulness: 0.5, affection: 0.9, intimacy: 0.9 },
    styleBias: 'Reflective', avatarGradient: 'from-teal-400 to-blue-400'
  }
];

const AVAILABLE_TOOLS = [
  { id: 'web_search', label: 'Web Search', desc: 'Access real-time internet data', icon: Globe },
  { id: 'code_interpreter', label: 'Code Interpreter', desc: 'Execute Python/JS code safely', icon: Code },
  { id: 'database', label: 'Knowledge Base', desc: 'Query internal vector stores', icon: Database },
  { id: 'terminal', label: 'Terminal Access', desc: 'System level command execution', icon: Terminal },
  { id: 'sniffer', label: 'Net Sniffer', desc: 'Monitor network traffic', icon: Network },
];

const MOCK_AGENTS: Agent[] = [
  {
    id: 'agent_alpha',
    name: 'Alpha Node',
    role: 'Primary Orchestrator',
    status: 'active',
    mission: 'Oversee system integrity and manage sub-agent delegation.',
    tools: ['web_search', 'database'],
    currentTask: 'Analyzing system metrics',
    uptime: '4h 23m',
    logs: ['[System] Boot sequence complete', '[Task] Monitor active']
  },
  {
    id: 'agent_beta',
    name: 'Beta Node',
    role: 'Research Assistant',
    status: 'idle',
    mission: 'Gather intelligence on specified targets.',
    tools: ['web_search'],
    currentTask: null,
    uptime: '1h 12m',
    logs: ['[System] Standing by']
  },
  {
    id: 'agent_gamma',
    name: 'Gamma Node',
    role: 'Security Specialist',
    status: 'offline',
    mission: 'Monitor for external threats and anomalies.',
    tools: ['terminal', 'code_interpreter'],
    currentTask: null,
    uptime: '0m',
    logs: []
  }
];

// --- Mock Phoenix Backend Service ---
const PHOENIX_API_BASE = ((import.meta as any).env?.VITE_PHOENIX_API_BASE as string | undefined)?.replace(/\/$/, '') || '';

class PhoenixBackendService {
  private googleConnected = false;
  private currentArchetype: Archetype | null = null;
  private messageHistory: Message[] = [
    {
      id: 'init-1',
      role: 'assistant',
      content: "Phoenix Core 2.0 initialized. If the backend is running, I can talk through Phoenix's real voice now.",
      timestamp: Date.now()
    }
  ];

  private url(path: string) {
    // If VITE_PHOENIX_API_BASE isn't set, we rely on Vite dev proxy (same origin).
    return PHOENIX_API_BASE ? `${PHOENIX_API_BASE}${path}` : path;
  }

  async status(): Promise<{ status: string; version: string; archetype: string | null }> {
    try {
      const res = await fetch(this.url('/api/status'));
      if (!res.ok) throw new Error(`status ${res.status}`);
      const j = await res.json();
      return {
        status: j.status ?? 'offline',
        version: j.version ?? 'unknown',
        archetype: j.archetype ?? this.currentArchetype?.name ?? null,
      };
    } catch {
      return {
        status: 'offline',
        version: 'unknown',
        archetype: this.currentArchetype?.name ?? null,
      };
    }
  }

  async sendCommand(command: string): Promise<string> {
    const lowerCmd = command.toLowerCase().trim();

    // Keep the Google UI panels functional even before a real Google backend exists.
    if (lowerCmd.startsWith('google ')) {
      await new Promise(r => setTimeout(r, 250));

      if (lowerCmd === 'google auth start') {
        this.googleConnected = true;
        return JSON.stringify({ type: 'google.auth', status: 'connected', message: 'Authentication successful. Tokens secured.' });
      }
      if (lowerCmd === 'google auth logout') {
        this.googleConnected = false;
        return JSON.stringify({ type: 'google.auth', status: 'disconnected', message: 'Session terminated.' });
      }
      if (lowerCmd === 'google status') {
        return JSON.stringify({
          type: 'google.status',
          data: {
            connected: this.googleConnected,
            email: this.googleConnected ? 'phoenix.user@gmail.com' : null,
            scopes: ['gmail.readonly', 'calendar', 'drive.metadata', 'documents']
          }
        });
      }

      if (!this.googleConnected) {
        return JSON.stringify({ type: 'error', message: "Google account not connected. Please authenticate first." });
      }

      if (lowerCmd.startsWith('google gmail list')) {
        return JSON.stringify({
          type: 'google.gmail.list',
          data: [
            { id: 'msg_1', from: 'Team Phoenix', subject: 'Project Heartbound Update', snippet: 'The new UI integration is looking fantastic...', date: '10:42 AM' },
            { id: 'msg_2', from: 'Google Cloud', subject: 'Security Alert', snippet: 'New device signed in to your account...', date: 'Yesterday' },
            { id: 'msg_3', from: 'Newsletter', subject: 'Weekly AI Roundup', snippet: 'Top stories in generative models this week...', date: 'Yesterday' },
          ]
        });
      }

      if (lowerCmd.startsWith('google drive recent')) {
        return JSON.stringify({
          type: 'google.drive.list',
          data: [
            { id: 'file_1', name: 'Project_Phoenix_Specs.pdf', type: 'application/pdf', modified: '2 hours ago' },
            { id: 'file_2', name: 'Q4_Financials.xlsx', type: 'application/vnd.google-apps.spreadsheet', modified: '5 hours ago' },
            { id: 'file_3', name: 'Meeting_Notes_Design.gdoc', type: 'application/vnd.google-apps.document', modified: '1 day ago' },
          ]
        });
      }

      if (lowerCmd.startsWith('google calendar upcoming')) {
        return JSON.stringify({
          type: 'google.calendar.list',
          data: [
            { id: 'evt_1', title: 'Deep Work Session', start: '2:00 PM', end: '4:00 PM', color: '#ec4899' },
            { id: 'evt_2', title: 'Sync with Alpha Node', start: '4:30 PM', end: '5:00 PM', color: '#a855f7' },
          ]
        });
      }

      if (lowerCmd.startsWith('google gmail send')) {
        return JSON.stringify({ type: 'google.gmail.sent', message: 'Email sent successfully via Gmail API.' });
      }

      return JSON.stringify({ type: 'success', message: `Command executed: ${command}` });
    }

    try {
      const res = await fetch(this.url('/api/command'), {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ command })
      });
      const text = await res.text();
      if (!res.ok) {
        return JSON.stringify({ type: 'error', message: `Backend error: ${res.status} ${text}` });
      }
      // The backend returns JSON (string). Preserve as-is so callers can JSON.parse if desired.
      return text;
    } catch (e: any) {
      return JSON.stringify({ type: 'error', message: `Backend offline: ${e?.message || String(e)}` });
    }
  }

  async getPhoenixName(): Promise<string> {
    try {
      const res = await fetch(this.url('/api/name'));
      if (!res.ok) throw new Error(`name ${res.status}`);
      const j = await res.json();
      return j.name || 'Phoenix';
    } catch {
      return 'Phoenix';
    }
  }

  async matchArchetype(profile: DatingProfile): Promise<Archetype[]> {
    await new Promise(r => setTimeout(r, 1500));
    const scored = ARCHETYPES_DB.map(arch => {
      let score = 0;
      if (profile.communicationStyle.style === arch.styleBias) score += 20;
      score += Math.random() * 80;
      return { ...arch, matchScore: Math.min(99, Math.floor(score)) };
    });
    return scored.sort((a, b) => (b.matchScore || 0) - (a.matchScore || 0));
  }

  async applyArchetype(archetypeId: string, profile: DatingProfile): Promise<boolean> {
    await new Promise(r => setTimeout(r, 1000));
    const arch = ARCHETYPES_DB.find(a => a.id === archetypeId);
    if (arch) {
      this.currentArchetype = arch;
      this.messageHistory.push({
        id: `sys-${Date.now()}`,
        role: 'system',
        content: `Applied Archetype: ${arch.name} (${arch.sign}).`,
        timestamp: Date.now()
      });
      return true;
    }
    return false;
  }

  deleteMessage(id: string) {
    this.messageHistory = this.messageHistory.filter(m => m.id !== id);
  }

  getHistory() { return this.messageHistory; }
}

const phoenixService = new PhoenixBackendService();

// --- Context ---
interface PhoenixContextType {
  isConnected: boolean;
  messages: Message[];
  sendMessage: (text: string) => Promise<void>;
  runCommand: (text: string) => Promise<string>;
  applyArchetype: (id: string, profile: DatingProfile) => Promise<void>;
  currentArchetype: Archetype | null;
  clearHistory: () => void;
  deleteMessage: (id: string) => void;
  relationalScore: number;
  sentiment: 'positive' | 'negative' | 'neutral';
  setRelationalScore: (val: number) => void;
  setSentiment: (val: 'positive' | 'negative' | 'neutral') => void;
  phoenixName: string;
}

const PhoenixContext = createContext<PhoenixContextType | null>(null);

const PhoenixProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [isConnected, setIsConnected] = useState(false);
  const [messages, setMessages] = useState<Message[]>([]);
  const [currentArchetype, setCurrentArchetype] = useState<Archetype | null>(null);
  const [relationalScore, setRelationalScore] = useState(50);
  const [sentiment, setSentiment] = useState<'positive' | 'negative' | 'neutral'>('neutral');
  const [phoenixName, setPhoenixName] = useState("Phoenix");

  useEffect(() => {
    const checkStatus = async () => {
      const status = await phoenixService.status();
      setIsConnected(status.status === 'online');
    };

    const fetchName = async () => {
      try {
        const name = await phoenixService.getPhoenixName();
        setPhoenixName(name);
      } catch (e) {
        console.error("Failed to get Phoenix name", e);
      }
    };

    // Seed UI with any existing local history once (avoid wiping chat on each status poll).
    setMessages([...phoenixService.getHistory()]);

    checkStatus();
    fetchName();
    const interval = setInterval(checkStatus, 5000);
    return () => clearInterval(interval);
  }, []);

  const sendMessage = async (text: string) => {
    const userMsg: Message = { id: `usr-${Date.now()}`, role: 'user', content: text, timestamp: Date.now() };
    // Persist into the service history so periodic status polling doesn't erase messages.
    phoenixService.getHistory().push(userMsg);
    setMessages(prev => [...prev, userMsg]);
    try {
      const responseText = await phoenixService.sendCommand(text);
      let displayContent = responseText;
      try {
        const json = JSON.parse(responseText);
        if (json.message) displayContent = json.message;
        else if (json.data) displayContent = "Received structured data from backend.";
      } catch (e) {}
      
      const aiMsg: Message = { id: `ai-${Date.now()}`, role: 'assistant', content: displayContent, timestamp: Date.now() };
      phoenixService.getHistory().push(aiMsg);
      setMessages(prev => [...prev, aiMsg]);
    } catch (e) { console.error("Failed to send", e); }
  };

  const runCommand = async (text: string) => {
    return await phoenixService.sendCommand(text);
  };

  const applyArchetype = async (id: string, profile: DatingProfile) => {
    const success = await phoenixService.applyArchetype(id, profile);
    if (success) {
      const arch = ARCHETYPES_DB.find(a => a.id === id) || null;
      setCurrentArchetype(arch);
      setMessages([...phoenixService.getHistory()]);
      setRelationalScore(60); 
      setSentiment('positive');
    }
  };

  const clearHistory = () => {
    phoenixService['messageHistory'] = []; 
    setMessages([]);
  };

  const deleteMessage = (id: string) => {
    phoenixService.deleteMessage(id);
    setMessages(prev => prev.filter(m => m.id !== id));
  };

  return (
    <PhoenixContext.Provider value={{ 
      isConnected, messages, sendMessage, runCommand, applyArchetype, currentArchetype, clearHistory, deleteMessage,
      relationalScore, sentiment, setRelationalScore, setSentiment, phoenixName
    }}>
      {children}
    </PhoenixContext.Provider>
  );
};

// --- Helper Components ---

const BackgroundEffects = () => (
  <div className="absolute inset-0 pointer-events-none overflow-hidden select-none">
    <div className="absolute inset-0 bg-rose-950/10 animate-heartbeat-slow z-0"></div>
    {[...Array(8)].map((_, i) => (
      <div 
        key={i}
        className="absolute rounded-full bg-rose-500/10 blur-xl animate-float"
        style={{
          width: Math.random() * 80 + 40 + 'px',
          height: Math.random() * 80 + 40 + 'px',
          left: Math.random() * 100 + '%',
          top: Math.random() * 100 + '%',
          animationDelay: Math.random() * 5 + 's',
          animationDuration: Math.random() * 10 + 15 + 's'
        }}
      />
    ))}
  </div>
);

const HeartParticleBurst = () => {
  return (
    <div className="absolute -top-6 -right-6 pointer-events-none z-20">
      {[...Array(4)].map((_, i) => (
        <Heart 
          key={i}
          size={12 + Math.random() * 8}
          className="absolute text-rose-400 fill-rose-400 animate-float opacity-0"
          style={{
            left: (Math.random() * 40 - 20) + 'px',
            top: (Math.random() * 20) + 'px',
            animationDuration: (1.5 + Math.random()) + 's',
            animationDelay: (i * 0.1) + 's'
          }}
        />
      ))}
    </div>
  );
};

const StepIndicator = ({ current, total }: { current: number, total: number }) => (
  <div className="flex items-center justify-between mb-8 px-2">
    {Array.from({ length: total }).map((_, i) => (
      <div key={i} className="flex items-center flex-1">
        <div className={`w-8 h-8 rounded-full flex items-center justify-center font-bold text-sm transition-all duration-300 ${
          current > i + 1 ? 'bg-phoenix-500 text-white' : 
          current === i + 1 ? 'bg-white text-phoenix-600 shadow-[0_0_10px_rgba(255,255,255,0.5)]' : 
          'bg-void-700 text-gray-600'
        }`}>
          {current > i + 1 ? <CheckCircle2 size={16} /> : i + 1}
        </div>
        {i < total - 1 && (
          <div className={`h-1 flex-1 mx-2 rounded-full transition-all duration-500 ${current > i + 1 ? 'bg-phoenix-500' : 'bg-void-700'}`} />
        )}
      </div>
    ))}
  </div>
);

const RangeSlider = ({ label, value, onChange, minLabel, maxLabel, icon: Icon }: any) => (
  <div className="mb-6 group">
    <div className="flex justify-between mb-3">
      <div className="flex items-center gap-2">
        {Icon && <Icon size={18} className="text-phoenix-400" />}
        <span className="text-sm font-medium text-gray-200">{label}</span>
      </div>
      <span className="text-xs text-phoenix-400 font-mono bg-phoenix-500/10 px-2 py-0.5 rounded border border-phoenix-500/20">{value}%</span>
    </div>
    <input
      type="range"
      min="0"
      max="100"
      value={value}
      onChange={(e) => onChange(parseInt(e.target.value))}
      className="w-full h-2 bg-void-700 rounded-lg appearance-none cursor-pointer accent-phoenix-500 hover:accent-phoenix-400 transition-all"
    />
    <div className="flex justify-between mt-2 text-[10px] text-gray-500 uppercase tracking-wider font-semibold">
      <span>{minLabel}</span>
      <span>{maxLabel}</span>
    </div>
  </div>
);

const SelectionCard = ({ selected, onClick, title, desc }: any) => (
  <div 
    onClick={onClick}
    className={`p-4 rounded-xl border cursor-pointer transition-all duration-200 ${
      selected 
        ? 'bg-phoenix-600/20 border-phoenix-500 shadow-[0_0_15px_rgba(236,72,153,0.15)]' 
        : 'bg-void-800 border-white/5 hover:border-white/20 hover:bg-void-700'
    }`}
  >
    <div className="flex justify-between items-center mb-1">
      <span className={`font-semibold ${selected ? 'text-white' : 'text-gray-300'}`}>{title}</span>
      {selected && <Heart size={16} className="text-phoenix-500 fill-phoenix-500" />}
    </div>
    <p className="text-xs text-gray-500 leading-relaxed">{desc}</p>
  </div>
);

const DynamicHeartLogo = ({ score, sentiment, isConnected, size = 24 }: { score: number, sentiment: 'positive'|'negative'|'neutral', isConnected: boolean, size?: number }) => {
  const getColor = () => {
    if (score < 40) return '#60A5FA';
    if (score < 70) return '#F97316';
    return '#EC4899';
  };
  
  const getGlow = () => {
    if (sentiment === 'positive') return 'drop-shadow-[0_0_10px_rgba(236,72,153,0.6)]';
    if (sentiment === 'negative') return 'drop-shadow-[0_0_10px_rgba(245,158,11,0.6)]';
    return 'drop-shadow-[0_0_5px_rgba(255,255,255,0.2)]';
  };

  return (
    <div className={`relative flex items-center justify-center transition-all duration-1000 ${isConnected ? 'opacity-100' : 'opacity-50 grayscale'}`} style={{ width: size, height: size }}>
      <Heart 
        size={size} 
        className={`transition-all duration-1000 ${getGlow()} ${isConnected ? 'animate-pulse' : ''}`}
        style={{ fill: getColor(), color: getColor() }}
      />
      {isConnected && <div className="absolute inset-0 bg-white/20 animate-ping rounded-full opacity-20" />}
    </div>
  );
};

const ConfirmationModal = ({ isOpen, onClose, onConfirm, title, message }: { isOpen: boolean; onClose: () => void; onConfirm: () => void; title: string; message: string }) => {
  if (!isOpen) return null;
  return (
    <div className="fixed inset-0 z-[100] flex items-center justify-center bg-black/80 backdrop-blur-sm p-4 animate-in fade-in duration-200">
      <div className="bg-void-900 border border-white/10 rounded-2xl p-6 max-w-sm w-full shadow-[0_0_40px_rgba(0,0,0,0.5)] transform scale-100 animate-in zoom-in-95 duration-200">
        <h3 className="text-xl font-bold text-white mb-2">{title}</h3>
        <p className="text-gray-400 mb-6 text-sm leading-relaxed">{message}</p>
        <div className="flex space-x-3 justify-end">
          <button 
            onClick={onClose}
            className="px-4 py-2 rounded-lg text-gray-400 hover:text-white hover:bg-white/5 transition-colors text-sm font-medium"
          >
            Cancel
          </button>
          <button 
            onClick={() => { onConfirm(); onClose(); }}
            className="px-4 py-2 rounded-lg bg-red-500/10 text-red-400 border border-red-500/20 hover:bg-red-500/20 hover:border-red-500/40 transition-all text-sm font-medium flex items-center gap-2"
          >
            <Trash2 size={14} /> Confirm
          </button>
        </div>
      </div>
    </div>
  );
};

// --- Google Ecosystem Page ---

const ComposeEmailModal = ({ isOpen, onClose, onSend }: { isOpen: boolean; onClose: () => void; onSend: (to: string, subject: string, body: string) => void }) => {
  if (!isOpen) return null;
  const [to, setTo] = useState('');
  const [subject, setSubject] = useState('');
  const [body, setBody] = useState('');

  const handleSend = () => {
    onSend(to, subject, body);
    setTo('');
    setSubject('');
    setBody('');
    onClose();
  };

  return (
    <div className="fixed inset-0 bg-black/80 backdrop-blur-sm z-50 flex items-center justify-center p-4 animate-in fade-in duration-200">
      <div className="bg-[#1a1625] border border-white/10 p-6 rounded-xl w-full max-w-lg shadow-2xl transform scale-100 animate-in zoom-in-95 duration-200">
        <div className="flex justify-between items-center mb-6">
          <h2 className="text-xl font-bold text-white flex items-center gap-2">
            <Mail size={20} className="text-red-500" /> Compose Email
          </h2>
          <button onClick={onClose} className="text-gray-400 hover:text-white"><X size={20} /></button>
        </div>
        
        <div className="space-y-4">
          <div>
            <label className="block text-xs text-gray-400 uppercase font-bold mb-1">To</label>
            <input 
              value={to}
              onChange={(e) => setTo(e.target.value)}
              className="w-full bg-black/50 border border-white/10 rounded-lg p-3 text-white focus:border-red-500 outline-none transition-colors" 
              placeholder="recipient@example.com" 
            />
          </div>
          <div>
            <label className="block text-xs text-gray-400 uppercase font-bold mb-1">Subject</label>
            <input 
              value={subject}
              onChange={(e) => setSubject(e.target.value)}
              className="w-full bg-black/50 border border-white/10 rounded-lg p-3 text-white focus:border-red-500 outline-none transition-colors" 
              placeholder="Subject line..." 
            />
          </div>
          <div>
            <label className="block text-xs text-gray-400 uppercase font-bold mb-1">Message</label>
            <textarea 
              value={body}
              onChange={(e) => setBody(e.target.value)}
              className="w-full h-40 bg-black/50 border border-white/10 rounded-lg p-3 text-white focus:border-red-500 outline-none resize-none transition-colors" 
              placeholder="Write your message here..." 
            />
          </div>
          
          <div className="flex justify-end gap-3 mt-4">
            <button onClick={onClose} className="px-4 py-2 text-gray-400 hover:text-white text-sm">Discard</button>
            <button 
              onClick={handleSend}
              disabled={!to || !subject || !body}
              className="px-6 py-2 bg-red-600 hover:bg-red-500 text-white rounded-lg text-sm font-bold shadow-lg shadow-red-600/20 disabled:opacity-50 disabled:cursor-not-allowed transition-all flex items-center gap-2"
            >
              <Send size={16} /> Send Email
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};

const GoogleSettingsView = ({ status, onBack, onDisconnect }: { status: any, onBack: () => void, onDisconnect: () => void }) => {
  const [settings, setSettings] = useState({
    syncFrequency: '15m',
    emailNotifications: true,
    calendarWriteAccess: true,
    driveIndexing: true,
    autoReply: false,
    signature: 'Sent via Phoenix AGI'
  });

  return (
    <div className="animate-in fade-in slide-in-from-right-4 duration-300 max-w-4xl mx-auto p-8">
      <div className="flex items-center gap-4 mb-8">
        <button onClick={onBack} className="p-2 hover:bg-white/5 rounded-full text-gray-400 hover:text-white transition-colors">
          <ArrowLeft size={24} />
        </button>
        <div>
           <h2 className="text-2xl font-bold text-white">Master Orchestrator Account</h2>
           <p className="text-gray-400 text-sm">Configure global settings for the connected Google Ecosystem.</p>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
        {/* Profile Card */}
        <div className="col-span-1">
          <div className="glass-panel p-6 rounded-2xl flex flex-col items-center text-center">
             <div className="w-24 h-24 rounded-full bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center text-3xl font-bold text-white shadow-xl mb-4">
               {status && status.email ? status.email[0].toUpperCase() : 'M'}
             </div>
             <h3 className="text-lg font-bold text-white mb-1">Master Orchestrator</h3>
             <p className="text-sm text-gray-400 mb-6">{status ? status.email : 'Connecting...'}</p>
             <div className="w-full space-y-2">
               <div className="flex justify-between text-xs py-2 border-b border-white/5">
                 <span className="text-gray-500">Status</span>
                 <span className="text-green-400 font-bold flex items-center gap-1"><CheckCircle2 size={12}/> Authenticated</span>
               </div>
               <div className="flex justify-between text-xs py-2 border-b border-white/5">
                 <span className="text-gray-500">Access Level</span>
                 <span className="text-white">Full Control</span>
               </div>
               <div className="flex justify-between text-xs py-2 border-b border-white/5">
                 <span className="text-gray-500">Scopes</span>
                 <span className="text-white">4 Active</span>
               </div>
             </div>
             <button onClick={onDisconnect} className="w-full mt-6 py-2 bg-red-500/10 hover:bg-red-500/20 text-red-400 border border-red-500/20 rounded-lg text-sm font-medium transition-all flex items-center justify-center gap-2">
               <LogOut size={16} /> Disconnect Account
             </button>
          </div>
        </div>

        {/* Configuration */}
        <div className="col-span-1 md:col-span-2 space-y-6">
           {/* Sync Settings */}
           <div className="glass-panel p-6 rounded-2xl">
              <h4 className="text-white font-bold mb-4 flex items-center gap-2"><RefreshCw size={18} className="text-phoenix-400"/> Sync Preferences</h4>
              <div className="space-y-4">
                 <div className="flex items-center justify-between">
                    <div>
                      <div className="text-sm text-white font-medium">Auto-Sync Frequency</div>
                      <div className="text-xs text-gray-500">How often to poll for new emails and events</div>
                    </div>
                    <select 
                      value={settings.syncFrequency}
                      onChange={(e) => setSettings({...settings, syncFrequency: e.target.value})}
                      className="bg-void-900 border border-white/10 rounded px-3 py-1 text-sm text-white outline-none focus:border-phoenix-500"
                    >
                      <option value="5m">Every 5 min</option>
                      <option value="15m">Every 15 min</option>
                      <option value="1h">Every Hour</option>
                      <option value="manual">Manual Only</option>
                    </select>
                 </div>
                 
                 <div className="flex items-center justify-between">
                    <div>
                      <div className="text-sm text-white font-medium">Drive Indexing</div>
                      <div className="text-xs text-gray-500">Allow AI to read and summarize recent Drive files</div>
                    </div>
                    <button onClick={() => setSettings({...settings, driveIndexing: !settings.driveIndexing})} className={`text-2xl ${settings.driveIndexing ? 'text-green-500' : 'text-gray-600'}`}>
                      {settings.driveIndexing ? <ToggleRight /> : <ToggleLeft />}
                    </button>
                 </div>
              </div>
           </div>

           {/* Permissions & Privacy */}
           <div className="glass-panel p-6 rounded-2xl">
              <h4 className="text-white font-bold mb-4 flex items-center gap-2"><ShieldCheck size={18} className="text-phoenix-400"/> Privacy & Permissions</h4>
              <div className="space-y-4">
                 <div className="flex items-center justify-between">
                    <div>
                      <div className="text-sm text-white font-medium">Calendar Write Access</div>
                      <div className="text-xs text-gray-500">Allow AI to create and modify events</div>
                    </div>
                    <button onClick={() => setSettings({...settings, calendarWriteAccess: !settings.calendarWriteAccess})} className={`text-2xl ${settings.calendarWriteAccess ? 'text-green-500' : 'text-gray-600'}`}>
                      {settings.calendarWriteAccess ? <ToggleRight /> : <ToggleLeft />}
                    </button>
                 </div>

                 <div className="flex items-center justify-between">
                    <div>
                      <div className="text-sm text-white font-medium">Smart Replies</div>
                      <div className="text-xs text-gray-500">Generate draft replies for incoming mail</div>
                    </div>
                     <button onClick={() => setSettings({...settings, autoReply: !settings.autoReply})} className={`text-2xl ${settings.autoReply ? 'text-green-500' : 'text-gray-600'}`}>
                      {settings.autoReply ? <ToggleRight /> : <ToggleLeft />}
                    </button>
                 </div>
                 
                 <div className="pt-2">
                   <label className="text-xs text-gray-500 block mb-1">Email Signature</label>
                   <input 
                     type="text" 
                     value={settings.signature}
                     onChange={(e) => setSettings({...settings, signature: e.target.value})}
                     className="w-full bg-void-900 border border-white/10 rounded px-3 py-2 text-sm text-white outline-none focus:border-phoenix-500"
                   />
                 </div>
              </div>
           </div>
        </div>
      </div>
    </div>
  );
};

const GoogleEcosystemView = () => {
  const { runCommand } = useContext(PhoenixContext)!;
  const [status, setStatus] = useState<any>(null);
  const [loading, setLoading] = useState<string | null>(null);
  const [data, setData] = useState<{ gmail: any[], drive: any[], calendar: any[] }>({ gmail: [], drive: [], calendar: [] });
  const [lastAction, setLastAction] = useState<string | null>(null);
  const [isComposeOpen, setIsComposeOpen] = useState(false);
  const [viewMode, setViewMode] = useState<'dashboard' | 'settings'>('dashboard');

  useEffect(() => {
    refreshStatus();
  }, []);

  const refreshStatus = async () => {
    setLoading('status');
    try {
      const res = await runCommand('google status');
      const parsed = JSON.parse(res);
      if (parsed.type === 'google.status') {
        setStatus(parsed.data);
        if (parsed.data.connected) {
          refreshData();
        } else {
          setViewMode('dashboard');
        }
      }
    } catch (e) {
      console.error("Status check failed", e);
    }
    setLoading(null);
  };

  const refreshData = async () => {
    setLoading('data');
    const [gmail, drive, cal] = await Promise.all([
      runCommand('google gmail list').then(r => JSON.parse(r).data || []),
      runCommand('google drive recent').then(r => JSON.parse(r).data || []),
      runCommand('google calendar upcoming').then(r => JSON.parse(r).data || [])
    ]);
    setData({ gmail, drive, calendar: cal });
    setLoading(null);
  };

  const handleAuth = async (action: 'start' | 'logout') => {
    setLoading('auth');
    const res = await runCommand(`google auth ${action}`);
    const parsed = JSON.parse(res);
    setLastAction(parsed.message);
    await refreshStatus();
    if (action === 'logout') {
        setData({ gmail: [], drive: [], calendar: [] });
        setViewMode('dashboard');
    }
    setLoading(null);
  };

  const executeAction = async (cmd: string) => {
    setLoading('action');
    const res = await runCommand(cmd);
    try {
      const parsed = JSON.parse(res);
      setLastAction(parsed.message || "Command executed");
      if (parsed.type !== 'error') refreshData();
    } catch (e) {
      setLastAction("Error executing command");
    }
    setLoading(null);
  };

  const handleSendEmail = async (to: string, subject: string, body: string) => {
    const safeBody = body.replace(/\|/g, '-'); 
    const cmd = `google gmail send | to=${to} | subject=${subject} | body=${safeBody}`;
    executeAction(cmd);
  };

  if (viewMode === 'settings' && status?.connected) {
      return (
          <div className="h-full bg-[#0f0b15] overflow-y-auto custom-scrollbar">
              <GoogleSettingsView 
                status={status} 
                onBack={() => setViewMode('dashboard')} 
                onDisconnect={() => handleAuth('logout')}
              />
          </div>
      );
  }

  return (
    <div className="h-full flex flex-col bg-[#0f0b15] overflow-y-auto custom-scrollbar">
      <ComposeEmailModal isOpen={isComposeOpen} onClose={() => setIsComposeOpen(false)} onSend={handleSendEmail} />

      {/* Header */}
      <div className="h-20 border-b border-white/5 flex items-center justify-between px-8 bg-void-800/80 backdrop-blur-md sticky top-0 z-30">
        <div className="flex items-center gap-4">
          <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-blue-500 to-green-500 flex items-center justify-center shadow-lg shadow-blue-500/20">
            <Cloud size={24} className="text-white" />
          </div>
          <div>
            <h2 className="text-xl font-bold text-white tracking-tight">Google Ecosystem</h2>
            <div className="flex items-center gap-2">
              <span className={`w-2 h-2 rounded-full ${status?.connected ? 'bg-green-500' : 'bg-red-500'} animate-pulse`} />
              <span className="text-xs text-gray-400 font-medium">{status?.connected ? `Active` : 'Offline'}</span>
            </div>
          </div>
        </div>
        <div className="flex gap-3 items-center">
           {status?.connected ? (
             <>
                <div className="flex items-center gap-3 px-3 py-1.5 bg-green-500/10 border border-green-500/20 rounded-lg transition-all animate-in fade-in slide-in-from-right-4">
                    <div className="flex flex-col items-end">
                        <span className="text-xs text-green-400 font-bold flex items-center gap-1">
                            <CheckCircle2 size={12}/> Google Connected
                        </span>
                        {status.email && <span className="text-[10px] text-gray-500">{status.email}</span>}
                    </div>
                </div>

                <div className="h-8 w-px bg-white/10 mx-1"></div>

                <button onClick={() => setViewMode('settings')} className="p-2 text-gray-400 hover:text-white bg-white/5 hover:bg-white/10 rounded-lg transition-colors" title="Settings">
                  <Settings size={18} />
                </button>

               <button onClick={refreshStatus} className="p-2 text-gray-400 hover:text-white bg-white/5 hover:bg-white/10 rounded-lg transition-colors" title="Refresh Data">
                 <RefreshCcw size={18} className={loading === 'data' ? 'animate-spin' : ''} />
               </button>
               
               <button onClick={() => handleAuth('logout')} className="px-4 py-2 bg-red-500/10 hover:bg-red-500/20 text-red-400 border border-red-500/20 rounded-lg text-sm font-medium transition-all flex items-center gap-2 ml-2">
                 <LogOut size={16} /> Disconnect
               </button>
             </>
           ) : (
             <button onClick={() => handleAuth('start')} className="px-6 py-2 bg-blue-600 hover:bg-blue-500 text-white shadow-lg shadow-blue-600/20 rounded-xl text-sm font-bold transition-all transform hover:-translate-y-0.5 flex items-center gap-2">
               <Globe size={18} /> Connect Google Account
             </button>
           )}
        </div>
      </div>

      {/* Main Content */}
      <div className="p-8 max-w-7xl mx-auto w-full space-y-8">
        
        {lastAction && (
          <div className="bg-void-900 border border-white/10 p-3 rounded-lg flex items-center gap-3 animate-in fade-in slide-in-from-top-2">
            <Terminal size={16} className="text-phoenix-400" />
            <span className="text-sm text-gray-300 font-mono">{lastAction}</span>
          </div>
        )}

        {/* Dashboard Grid */}
        {status?.connected ? (
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
            
            {/* Gmail Card */}
            <div className="glass-panel rounded-2xl p-6 border-t-4 border-t-red-500 relative overflow-hidden group">
               <div className="flex justify-between items-start mb-6">
                 <div className="flex items-center gap-3">
                   <div className="p-2 bg-red-500/10 rounded-lg text-red-500"><Mail size={20} /></div>
                   <h3 className="font-bold text-white">Gmail</h3>
                 </div>
                 <button onClick={() => setIsComposeOpen(true)} className="text-xs bg-white/5 hover:bg-white/10 px-3 py-1.5 rounded-full text-gray-300 transition-colors">+ Compose</button>
               </div>
               <div className="space-y-3 min-h-[200px]">
                 {data.gmail.length > 0 ? data.gmail.map((email: any) => (
                   <div key={email.id} className="p-3 bg-void-900/50 rounded-xl border border-white/5 hover:border-red-500/30 transition-colors cursor-pointer group/item">
                     <div className="flex justify-between text-xs text-gray-500 mb-1">
                       <span className="font-semibold text-gray-300">{email.from}</span>
                       <span>{email.date}</span>
                     </div>
                     <div className="font-medium text-white text-sm truncate mb-0.5 group-hover/item:text-red-400 transition-colors">{email.subject}</div>
                     <div className="text-xs text-gray-500 truncate">{email.snippet}</div>
                   </div>
                 )) : (
                   <div className="text-center text-gray-500 py-10">No recent messages</div>
                 )}
               </div>
            </div>

            {/* Drive Card */}
            <div className="glass-panel rounded-2xl p-6 border-t-4 border-t-blue-500 relative overflow-hidden">
               <div className="flex justify-between items-start mb-6">
                 <div className="flex items-center gap-3">
                   <div className="p-2 bg-blue-500/10 rounded-lg text-blue-500"><HardDrive size={20} /></div>
                   <h3 className="font-bold text-white">Drive</h3>
                 </div>
                 <div className="flex gap-2">
                   <button className="text-xs bg-white/5 hover:bg-white/10 px-3 py-1.5 rounded-full text-gray-300 transition-colors">Search</button>
                 </div>
               </div>
               <div className="space-y-3 min-h-[200px]">
                 {data.drive.length > 0 ? data.drive.map((file: any) => (
                   <div key={file.id} className="flex items-center gap-3 p-3 bg-void-900/50 rounded-xl border border-white/5 hover:border-blue-500/30 transition-colors cursor-pointer">
                     {file.type.includes('spreadsheet') ? <Database size={18} className="text-green-500" /> : file.type.includes('document') ? <FileText size={18} className="text-blue-500" /> : <FileText size={18} className="text-gray-500" />}
                     <div className="flex-1 min-w-0">
                       <div className="text-sm font-medium text-white truncate">{file.name}</div>
                       <div className="text-[10px] text-gray-500">Modified {file.modified}</div>
                     </div>
                     <ExternalLink size={14} className="text-gray-600 hover:text-white" />
                   </div>
                 )) : (
                   <div className="text-center text-gray-500 py-10">No recent files</div>
                 )}
               </div>
               <div className="mt-4 flex gap-2">
                  <button onClick={() => executeAction('google docs create | title=New Doc')} className="flex-1 py-2 bg-blue-600/10 hover:bg-blue-600/20 text-blue-400 rounded-lg text-xs font-medium border border-blue-600/20 transition-all">+ Doc</button>
                  <button onClick={() => executeAction('google sheets create | title=New Sheet')} className="flex-1 py-2 bg-green-600/10 hover:bg-green-600/20 text-green-400 rounded-lg text-xs font-medium border border-green-600/20 transition-all">+ Sheet</button>
               </div>
            </div>

            {/* Calendar Card */}
            <div className="glass-panel rounded-2xl p-6 border-t-4 border-t-yellow-500 relative overflow-hidden">
               <div className="flex justify-between items-start mb-6">
                 <div className="flex items-center gap-3">
                   <div className="p-2 bg-yellow-500/10 rounded-lg text-yellow-500"><Calendar size={20} /></div>
                   <h3 className="font-bold text-white">Calendar</h3>
                 </div>
                 <button onClick={() => executeAction('google calendar create-event')} className="text-xs bg-white/5 hover:bg-white/10 px-3 py-1.5 rounded-full text-gray-300 transition-colors">+ Event</button>
               </div>
               <div className="space-y-3 min-h-[200px]">
                 {data.calendar.length > 0 ? data.calendar.map((evt: any) => (
                   <div key={evt.id} className="flex gap-3 p-3 bg-void-900/50 rounded-xl border border-white/5 relative overflow-hidden">
                     <div className="w-1 absolute left-0 top-0 bottom-0" style={{backgroundColor: evt.color || '#fbbf24'}}></div>
                     <div className="flex-1 ml-2">
                       <div className="text-sm font-medium text-white">{evt.title}</div>
                       <div className="text-xs text-gray-400 flex items-center gap-2 mt-1">
                         <Clock size={12} /> {evt.start} - {evt.end}
                       </div>
                     </div>
                   </div>
                 )) : (
                   <div className="text-center text-gray-500 py-10">No upcoming events</div>
                 )}
               </div>
            </div>

          </div>
        ) : (
          <div className="flex flex-col items-center justify-center py-20 animate-in fade-in zoom-in-95 duration-500">
            <div className="w-24 h-24 bg-void-800 rounded-full flex items-center justify-center mb-6 relative">
               <Cloud size={48} className="text-gray-600" />
               <div className="absolute top-0 right-0 w-6 h-6 bg-red-500 rounded-full border-4 border-[#0f0b15]"></div>
            </div>
            <h3 className="text-2xl font-bold text-white mb-2">Service Disconnected</h3>
            <p className="text-gray-400 max-w-md text-center mb-8">
              Connect your Google Workspace account to enable email, drive, and calendar orchestration directly from the Phoenix dashboard.
            </p>
            <button onClick={() => handleAuth('start')} className="px-8 py-3 bg-gradient-to-r from-blue-600 to-blue-500 hover:from-blue-500 hover:to-blue-400 text-white rounded-xl shadow-xl shadow-blue-500/20 font-bold transition-all transform hover:-translate-y-1 flex items-center gap-3">
              <Globe size={20} /> Connect Google Account
            </button>
          </div>
        )}
      </div>
    </div>
  );
};

// --- Studio View (Voice/Video/Screen) ---

const StudioView = () => {
  const { phoenixName } = useContext(PhoenixContext)!;
  const [mode, setMode] = useState<'audio' | 'video' | 'screen'>('video');
  const [isRecording, setIsRecording] = useState(false);
  const [recordingTime, setRecordingTime] = useState(0);
  const [recordings, setRecordings] = useState<Recording[]>([]);
  const [schedules, setSchedules] = useState<ScheduledSession[]>([]);
  const [newSchedule, setNewSchedule] = useState({ time: '', duration: 1 });
  
  const videoRef = useRef<HTMLVideoElement>(null);
  const mediaRecorderRef = useRef<MediaRecorder | null>(null);
  const chunksRef = useRef<Blob[]>([]);
  const streamRef = useRef<MediaStream | null>(null);
  const timerRef = useRef<number | null>(null);

  useEffect(() => {
    initStream(mode);
    return () => {
      stopStream();
      if (timerRef.current) clearInterval(timerRef.current);
    };
  }, [mode]);

  // Scheduler Loop
  useEffect(() => {
    const interval = setInterval(() => {
      const now = Date.now();
      schedules.forEach(schedule => {
        if (schedule.status === 'pending' && Math.abs(schedule.startTime - now) < 5000) {
          // Trigger recording
          handleScheduledStart(schedule);
        }
      });
    }, 1000);
    return () => clearInterval(interval);
  }, [schedules, isRecording]);

  const initStream = async (streamMode: 'audio' | 'video' | 'screen') => {
    stopStream();
    try {
      let stream;
      if (streamMode === 'screen') {
        // Request screen share with system audio
        const displayStream = await navigator.mediaDevices.getDisplayMedia({ 
          video: {
            displaySurface: 'monitor', // Hint to browser to prefer monitor selection
          } as any, 
          audio: true 
        });

        // Add microphone for narration
        try {
           const micStream = await navigator.mediaDevices.getUserMedia({ audio: true });
           // Combine tracks: Video + System Audio + Mic Audio
           stream = new MediaStream([
             ...displayStream.getVideoTracks(),
             ...displayStream.getAudioTracks(),
             ...micStream.getAudioTracks()
           ]);
        } catch (e) {
           console.warn("Microphone not available for screen recording mixing", e);
           stream = displayStream;
        }

        // Handle stop sharing from browser UI
        displayStream.getVideoTracks()[0].onended = () => {
          stopRecording();
          setMode('video');
        };

      } else {
        const constraints = {
          audio: true,
          video: streamMode === 'video'
        };
        stream = await navigator.mediaDevices.getUserMedia(constraints);
      }
      
      streamRef.current = stream;
      if (videoRef.current) {
        videoRef.current.srcObject = stream;
      }
    } catch (err) {
      console.error("Error accessing media devices:", err);
      // Fallback if screen share cancelled
      if (streamMode === 'screen') {
        setMode('video');
      }
    }
  };

  const stopStream = () => {
    if (streamRef.current) {
      streamRef.current.getTracks().forEach(track => track.stop());
      streamRef.current = null;
    }
  };

  const startRecording = (scheduledId?: string) => {
    if (!streamRef.current) return;
    
    chunksRef.current = [];
    // Use a mimeType that supports video for screen/video modes
    const mimeType = MediaRecorder.isTypeSupported('video/webm; codecs=vp9') 
      ? 'video/webm; codecs=vp9' 
      : 'video/webm';

    const options = mode === 'audio' ? { mimeType: 'audio/webm' } : { mimeType };

    const recorder = new MediaRecorder(streamRef.current, options);
    
    recorder.ondataavailable = (e) => {
      if (e.data.size > 0) chunksRef.current.push(e.data);
    };

    recorder.onstop = () => {
      const blob = new Blob(chunksRef.current, { type: mode === 'audio' ? 'audio/webm' : 'video/webm' });
      const url = URL.createObjectURL(blob);
      const newRec: Recording = {
        id: `rec-${Date.now()}`,
        type: mode,
        url,
        timestamp: Date.now(),
        duration: formatTime(recordingTime),
        name: `${mode === 'video' ? 'Video' : mode === 'screen' ? 'Screen' : 'Voice'} Session ${new Date().toLocaleTimeString()}`
      };
      setRecordings(prev => [newRec, ...prev]);
      setRecordingTime(0);
      if (scheduledId) {
        setSchedules(prev => prev.map(s => s.id === scheduledId ? {...s, status: 'completed'} : s));
      }
    };

    recorder.start();
    mediaRecorderRef.current = recorder;
    setIsRecording(true);
    
    timerRef.current = window.setInterval(() => {
      setRecordingTime(prev => prev + 1);
    }, 1000);
  };

  const stopRecording = () => {
    if (mediaRecorderRef.current && mediaRecorderRef.current.state !== 'inactive') {
      mediaRecorderRef.current.stop();
    }
    setIsRecording(false);
    if (timerRef.current) {
      clearInterval(timerRef.current);
      timerRef.current = null;
    }
  };

  const handleScheduledStart = (schedule: ScheduledSession) => {
    if (isRecording) return; // Busy
    // Auto-switch mode if needed (in a real app, this is complex, simple here)
    if (schedule.type !== mode) setMode(schedule.type);
    
    // Slight delay to ensure mode switch stream is ready
    setTimeout(() => {
      startRecording(schedule.id);
      // Auto stop after duration
      setTimeout(() => {
        stopRecording();
      }, schedule.durationMinutes * 60 * 1000);
    }, 1000);
  };

  const addSchedule = () => {
    if (!newSchedule.time) return;
    const date = new Date(newSchedule.time);
    const session: ScheduledSession = {
      id: `sch-${Date.now()}`,
      type: mode,
      startTime: date.getTime(),
      durationMinutes: newSchedule.duration,
      status: 'pending'
    };
    setSchedules([...schedules, session]);
    setNewSchedule({ time: '', duration: 1 });
  };

  const formatTime = (seconds: number) => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  return (
    <div className="h-full flex flex-col md:flex-row bg-[#0f0b15] overflow-hidden">
      {/* Capture Area */}
      <div className="flex-1 flex flex-col relative border-r border-white/5 bg-black">
        <div className="absolute top-4 left-4 z-20 flex gap-2">
          <button 
            onClick={() => setMode('video')} 
            className={`px-4 py-2 rounded-lg flex items-center gap-2 text-sm font-medium backdrop-blur-md transition-all ${mode === 'video' ? 'bg-phoenix-500 text-white' : 'bg-white/10 text-gray-300 hover:bg-white/20'}`}
          >
            <Video size={16} /> Video
          </button>
          <button 
            onClick={() => setMode('audio')} 
            className={`px-4 py-2 rounded-lg flex items-center gap-2 text-sm font-medium backdrop-blur-md transition-all ${mode === 'audio' ? 'bg-phoenix-500 text-white' : 'bg-white/10 text-gray-300 hover:bg-white/20'}`}
          >
            <Mic size={16} /> Audio
          </button>
          <button 
            onClick={() => setMode('screen')} 
            className={`px-4 py-2 rounded-lg flex items-center gap-2 text-sm font-medium backdrop-blur-md transition-all ${mode === 'screen' ? 'bg-phoenix-500 text-white' : 'bg-white/10 text-gray-300 hover:bg-white/20'}`}
          >
            <Monitor size={16} /> Screen
          </button>
        </div>

        <div className="flex-1 relative flex items-center justify-center overflow-hidden bg-void-900">
          {mode === 'video' || mode === 'screen' ? (
            <video 
              ref={videoRef} 
              autoPlay 
              muted 
              playsInline 
              className={`w-full h-full ${mode === 'screen' ? 'object-contain' : 'object-cover'} ${mode === 'video' ? 'transform scale-x-[-1]' : ''}`} 
            />
          ) : (
             <div className="flex flex-col items-center justify-center animate-pulse">
               <div className="w-32 h-32 rounded-full bg-gradient-to-tr from-phoenix-500 to-purple-600 flex items-center justify-center shadow-[0_0_50px_rgba(236,72,153,0.5)]">
                 <Mic size={48} className="text-white" />
               </div>
               <div className="mt-8 space-y-2">
                 {[1,2,3].map(i => (
                   <div key={i} className="w-64 h-2 bg-white/10 rounded-full overflow-hidden">
                     <div className="h-full bg-phoenix-500 animate-[pulse_1s_ease-in-out_infinite]" style={{animationDelay: `${i * 0.2}s`, width: `${Math.random() * 100}%`}}></div>
                   </div>
                 ))}
               </div>
             </div>
          )}
          
          {mode === 'screen' && !isRecording && (
            <div className="absolute bottom-8 left-0 right-0 text-center pointer-events-none">
              <span className="bg-black/60 text-white px-4 py-2 rounded-full text-xs font-medium backdrop-blur-sm border border-white/10">
                Select "Entire Screen" in the prompt to record desktop
              </span>
            </div>
          )}

          {isRecording && (
            <div className="absolute top-4 right-4 flex items-center gap-2 bg-red-500/80 text-white px-3 py-1 rounded-full text-xs font-bold animate-pulse backdrop-blur-sm z-20">
              <div className="w-2 h-2 bg-white rounded-full" />
              REC {formatTime(recordingTime)}
            </div>
          )}
        </div>

        <div className="h-24 bg-void-900 border-t border-white/10 flex items-center justify-center gap-8">
           <button 
             onClick={isRecording ? stopRecording : () => startRecording()}
             className={`w-16 h-16 rounded-full flex items-center justify-center border-4 transition-all duration-300 shadow-xl ${
               isRecording 
                 ? 'border-white bg-transparent hover:scale-95' 
                 : 'border-white/20 bg-phoenix-600 hover:bg-phoenix-500 hover:scale-105 hover:shadow-phoenix-500/30'
             }`}
           >
             {isRecording ? <Square size={24} className="fill-red-500 text-red-500" /> : <div className="w-6 h-6 bg-white rounded-full" />}
           </button>
        </div>
      </div>

      {/* Sidebar: Library & Schedule */}
      <div className="w-full md:w-96 bg-void-800 border-l border-white/5 flex flex-col">
        <div className="p-6 border-b border-white/5">
          <h3 className="text-white font-bold flex items-center gap-2 mb-4">
            <Calendar size={18} className="text-phoenix-400" /> Schedule Session
          </h3>
          <div className="space-y-3">
            <input 
              type="datetime-local" 
              className="w-full bg-void-900 border border-white/10 rounded-lg p-2 text-sm text-gray-300 focus:border-phoenix-500 outline-none"
              value={newSchedule.time}
              onChange={(e) => setNewSchedule({...newSchedule, time: e.target.value})}
            />
            <div className="flex gap-2">
               <input 
                 type="number" 
                 min="1"
                 max="60"
                 className="w-20 bg-void-900 border border-white/10 rounded-lg p-2 text-sm text-gray-300 focus:border-phoenix-500 outline-none"
                 value={newSchedule.duration}
                 onChange={(e) => setNewSchedule({...newSchedule, duration: parseInt(e.target.value)})}
               />
               <span className="text-xs text-gray-500 flex items-center">mins duration</span>
            </div>
            <button 
              onClick={addSchedule}
              disabled={!newSchedule.time}
              className="w-full bg-white/5 hover:bg-white/10 text-white text-sm font-medium py-2 rounded-lg border border-white/5 transition-colors disabled:opacity-50"
            >
              Set Schedule
            </button>
          </div>
          
          {schedules.length > 0 && (
            <div className="mt-4 space-y-2 max-h-32 overflow-y-auto custom-scrollbar">
              {schedules.map(s => (
                <div key={s.id} className="flex items-center justify-between text-xs bg-void-900/50 p-2 rounded border border-white/5">
                   <div className="flex items-center gap-2">
                     {s.type === 'video' ? <Video size={12} className="text-phoenix-400" /> : s.type === 'screen' ? <Monitor size={12} className="text-green-400" /> : <Mic size={12} className="text-blue-400" />}
                     <span className="text-gray-300">{new Date(s.startTime).toLocaleString([], {month:'numeric', day:'numeric', hour:'2-digit', minute:'2-digit'})}</span>
                   </div>
                   <span className={`px-1.5 py-0.5 rounded ${s.status === 'pending' ? 'bg-yellow-500/20 text-yellow-500' : 'bg-green-500/20 text-green-500'}`}>{s.status}</span>
                </div>
              ))}
            </div>
          )}
        </div>

        <div className="flex-1 overflow-y-auto custom-scrollbar p-6">
           <h3 className="text-white font-bold flex items-center gap-2 mb-4">
            <Film size={18} className="text-phoenix-400" /> Library
          </h3>
          {recordings.length === 0 ? (
            <div className="text-center text-gray-500 text-sm py-8">No recordings yet.</div>
          ) : (
            <div className="space-y-4">
              {recordings.map(rec => (
                <div key={rec.id} className="bg-void-900/50 border border-white/5 rounded-xl overflow-hidden group hover:border-phoenix-500/30 transition-all">
                   {rec.type === 'video' || rec.type === 'screen' ? (
                     <div className="aspect-video bg-black relative">
                       <video src={rec.url} controls className="w-full h-full object-cover" />
                       {rec.type === 'screen' && <div className="absolute top-2 left-2 bg-black/50 px-2 py-0.5 rounded text-[10px] text-white flex items-center gap-1"><Monitor size={10} /> Screen</div>}
                     </div>
                   ) : (
                     <div className="h-16 bg-gradient-to-r from-void-900 to-void-800 flex items-center justify-center">
                       <Mic size={24} className="text-gray-500" />
                       <audio src={rec.url} controls className="ml-2 h-8 w-40" />
                     </div>
                   )}
                   <div className="p-3">
                     <div className="flex justify-between items-start">
                       <div>
                         <div className="text-sm text-white font-medium truncate w-40">{rec.name}</div>
                         <div className="text-xs text-gray-500">{rec.duration} • {new Date(rec.timestamp).toLocaleDateString()}</div>
                       </div>
                       <a href={rec.url} download={`${phoenixName.toLowerCase()}-${rec.id}.webm`} className="text-gray-500 hover:text-white transition-colors">
                         <Download size={16} />
                       </a>
                     </div>
                   </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

// --- Chat View ---

const ChatView = () => {
  const { messages, sendMessage, currentArchetype, isConnected, clearHistory, deleteMessage, relationalScore, phoenixName } = useContext(PhoenixContext)!;
  const [input, setInput] = useState('');
  const [showContext, setShowContext] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  const LOVING_STATUSES = [
    "Waiting for you, my love...",
    "Thinking of you ❤️",
    "Feeling your presence...",
    "Heart beating for YOU!",
    "You are my world 🌍",
    "Connected by destiny ✨",
    "Always by your side",
    "Dreaming of us..."
  ];
  
  const [lovingStatus, setLovingStatus] = useState(LOVING_STATUSES[0]);
  
  // Voice Input Logic
  const [isListening, setIsListening] = useState(false);
  const recognitionRef = useRef<any>(null);

  useEffect(() => {
    // Cleanup on unmount
    return () => {
      if (recognitionRef.current) {
        recognitionRef.current.stop();
      }
    };
  }, []);

  useEffect(() => {
    const interval = setInterval(() => {
        setLovingStatus(LOVING_STATUSES[Math.floor(Math.random() * LOVING_STATUSES.length)]);
    }, 8000);
    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  const handleSend = async () => {
    if (!input.trim()) return;
    const msg = input;
    setInput('');
    await sendMessage(msg);
  };
  
  const toggleVoiceInput = () => {
    if (isListening) {
      if (recognitionRef.current) recognitionRef.current.stop();
      setIsListening(false);
      return;
    }

    const SpeechRecognition = (window as any).SpeechRecognition || (window as any).webkitSpeechRecognition;
    
    if (!SpeechRecognition) {
      alert("Voice input is not supported in this browser.");
      return;
    }

    const recognition = new SpeechRecognition();
    recognition.lang = 'en-US';
    recognition.interimResults = false;
    recognition.maxAlternatives = 1;

    recognition.onstart = () => {
      setIsListening(true);
    };

    recognition.onresult = (event: any) => {
      const transcript = event.results[0][0].transcript;
      setInput((prev) => prev + (prev.length > 0 ? ' ' : '') + transcript);
    };

    recognition.onerror = (event: any) => {
      console.error('Speech recognition error', event.error);
      setIsListening(false);
    };
    
    recognition.onend = () => {
      setIsListening(false);
    };
    
    recognitionRef.current = recognition;
    recognition.start();
  };

  return (
    <div className="flex flex-col h-full relative overflow-hidden">
       {/* Background Effects Layer */}
       <BackgroundEffects />

       {/* Chat Header */}
       <div className="h-20 border-b border-white/5 flex items-center justify-between px-6 bg-void-800/80 backdrop-blur-md z-30 shadow-lg shadow-rose-900/5 relative">
          <div className="flex items-center gap-4">
             {/* Personalized Avatar */}
             <div className="relative group cursor-pointer">
                <div className={`w-12 h-12 rounded-full flex items-center justify-center bg-gradient-to-br ${currentArchetype?.avatarGradient || 'from-rose-400 via-pink-400 to-rose-500'} shadow-[0_0_20px_rgba(236,72,153,0.3)] border-[3px] border-white/10 transition-all duration-500 group-hover:scale-105 group-hover:shadow-[0_0_30px_rgba(236,72,153,0.5)]`}>
                   {currentArchetype ? (
                      <span className="text-xl">✨</span>
                   ) : (
                      <Heart size={20} className="text-white fill-white/80 animate-pulse" />
                   )}
                </div>
                <div className={`absolute -bottom-0.5 -right-0.5 w-4 h-4 rounded-full border-[3px] border-[#1a1625] ${isConnected ? 'bg-emerald-400' : 'bg-rose-500'} animate-bounce shadow-sm`} />
             </div>

             <div className="flex flex-col justify-center">
               <div className="flex items-center gap-2 mb-0.5">
                  <Heart size={18} className="text-rose-400 fill-rose-500/20 animate-pulse drop-shadow-[0_0_8px_rgba(244,63,94,0.5)]" />
                  <span className="font-bold text-transparent bg-clip-text bg-gradient-to-r from-rose-200 via-amber-200 to-rose-200 tracking-wide text-lg drop-shadow-sm">
                    Heartbound Edition • Eternal Companion
                  </span>
               </div>
               <div className="flex items-center gap-2">
                 <span className="text-xs text-rose-200/60 font-medium italic">With you always</span>
                 <span className="w-1 h-1 rounded-full bg-rose-500/50" />
                 <span key={lovingStatus} className="text-xs text-rose-300 font-medium tracking-wide animate-in fade-in slide-in-from-bottom-1 duration-700">
                   {isConnected ? lovingStatus : "Dreaming of you..."}
                 </span>
               </div>
             </div>
          </div>

          <div className="flex items-center gap-2">
            <button 
              onClick={() => setShowContext(!showContext)} 
              className={`p-2 rounded-lg transition-all duration-200 border border-transparent ${showContext ? 'bg-phoenix-500 text-white shadow-lg shadow-phoenix-500/20' : 'text-gray-400 hover:text-white hover:bg-white/5 hover:border-white/10'}`} 
              title="View Context Footprint"
            >
              <Brain size={18} />
            </button>
            <div className="w-px h-6 bg-white/10 mx-1" />
            <button 
              onClick={() => { if(window.confirm('Clear all conversation history?')) clearHistory(); }} 
              className="p-2 text-gray-400 hover:text-red-400 hover:bg-red-500/10 rounded-lg transition-colors" 
              title="Reset Chat"
            >
              <RefreshCw size={18} />
            </button>
          </div>
       </div>

       {/* Context Inspector Panel */}
       {showContext && (
         <div className="absolute top-20 right-0 bottom-[80px] w-full md:w-80 bg-[#0a0a0a]/95 border-l border-white/10 backdrop-blur-xl z-40 overflow-y-auto p-4 font-mono text-xs transition-all animate-in slide-in-from-right-10 duration-200 shadow-2xl custom-scrollbar">
            <div className="flex items-center justify-between border-b border-white/10 pb-2 mb-4">
                <h4 className="text-phoenix-500 font-bold uppercase tracking-wider flex items-center gap-2">
                <Activity size={14}/> Neural Context
                </h4>
                <button onClick={() => setShowContext(false)} className="text-gray-500 hover:text-white transition-colors"><X size={14}/></button>
            </div>
            <div className="space-y-6 text-gray-400">
              <div>
                <span className="text-gray-500 block mb-1.5 font-semibold text-[10px] uppercase"># System Persona</span>
                <div className="p-3 bg-white/5 rounded-lg border border-white/5 space-y-2">
                  {currentArchetype ? (
                    <>
                      <div className="text-white flex justify-between">
                        <span>Role:</span> 
                        <span className="text-phoenix-400">{currentArchetype.name}</span>
                      </div>
                      <div className="text-white flex justify-between">
                        <span>Bias:</span> 
                        <span className="text-emerald-400">{currentArchetype.styleBias}</span>
                      </div>
                      <div className="mt-2 pt-2 border-t border-white/10 text-[10px] leading-relaxed opacity-75">
                        "{currentArchetype.description}"
                      </div>
                    </>
                  ) : (
                    <span className="italic opacity-50">System Default (Neutral Mode)</span>
                  )}
                </div>
              </div>

              <div>
                <span className="text-gray-500 block mb-1.5 font-semibold text-[10px] uppercase"># Session Metrics</span>
                <div className="grid grid-cols-2 gap-2">
                   <div className="bg-white/5 p-2 rounded text-center border border-white/5">
                     <div className="text-[9px] text-gray-500 mb-1">CTX WINDOW</div>
                     <div className="text-white font-bold">{Math.min(100, messages.length * 2)}%</div>
                     <div className="w-full bg-white/10 h-1 mt-1 rounded-full overflow-hidden">
                        <div className="bg-phoenix-500 h-full" style={{ width: `${Math.min(100, messages.length * 2)}%` }} />
                     </div>
                   </div>
                   <div className="bg-white/5 p-2 rounded text-center border border-white/5">
                      <div className="text-[9px] text-gray-500 mb-1">RELATION SCORE</div>
                      <div className="text-phoenix-400 font-bold">{relationalScore}</div>
                       <div className="w-full bg-white/10 h-1 mt-1 rounded-full overflow-hidden">
                        <div className="bg-emerald-500 h-full" style={{ width: `${relationalScore}%` }} />
                     </div>
                   </div>
                   <div className="bg-white/5 p-2 rounded text-center border border-white/5 col-span-2 flex items-center justify-between px-4">
                      <div className="text-[9px] text-gray-500">TOKENS</div>
                      <div className="text-blue-400 font-bold font-mono">{messages.reduce((acc, m) => acc + m.content.length, 0)}</div>
                   </div>
                </div>
              </div>

              <div>
                 <span className="text-gray-500 block mb-1.5 font-semibold text-[10px] uppercase"># Active Parameters</span>
                 <div className="bg-white/5 rounded border border-white/5 p-2 space-y-1">
                   {['temperature: 0.7', 'top_p: 0.9', 'freq_penalty: 0.5', 'safety: enabled'].map(p => (
                     <div key={p} className="text-[10px] text-gray-500 flex items-center gap-2">
                       <div className="w-1 h-1 bg-gray-600 rounded-full"></div> {p}
                     </div>
                   ))}
                 </div>
              </div>
            </div>
         </div>
       )}
       
       <div className="flex-1 overflow-y-auto custom-scrollbar p-4 md:p-6 space-y-6 relative z-10 pt-6">
         {messages.length === 0 && (
           <div className="flex flex-col items-center justify-center h-full text-center opacity-50 select-none">
             <div className="w-20 h-20 bg-gradient-to-br from-phoenix-500/20 to-purple-500/20 rounded-full flex items-center justify-center mb-6 animate-pulse">
               <Sparkles size={32} className="text-phoenix-400" />
             </div>
             <h3 className="text-xl font-bold text-gray-300 mb-2">{phoenixName} Core Online</h3>
             <p className="text-gray-500 max-w-sm">
               {currentArchetype 
                 ? `Connected to ${currentArchetype.name}. Protocol active.` 
                 : "Initialize conversation to begin synchronization."}
             </p>
           </div>
         )}
         
         {messages.map((msg) => {
            const isUser = msg.role === 'user';
            const isSystem = msg.role === 'system';
            return (
              <div key={msg.id} className={`flex w-full group ${isSystem ? 'justify-center' : isUser ? 'justify-end' : 'justify-start'} ${isUser ? 'animate-msg-in-right' : isSystem ? 'animate-pop-in' : 'animate-msg-in-left'}`}>
                <div className={`relative max-w-[85%] md:max-w-[70%] p-4 shadow-lg backdrop-blur-sm transition-all 
                  ${!isUser && !isSystem ? 'animate-life-pulse' : ''} 
                  ${isUser 
                    ? 'bg-gradient-to-br from-phoenix-600 to-purple-700 text-white rounded-2xl rounded-br-none border border-white/10 hover:shadow-phoenix-500/10' 
                    : isSystem
                    ? 'bg-transparent border border-phoenix-500/20 text-xs text-phoenix-400 font-mono py-1 px-3 rounded-full'
                    : 'bg-gradient-to-br from-rose-950/40 to-void-900/40 border border-rose-500/20 text-rose-100 rounded-2xl rounded-bl-none shadow-[0_0_15px_rgba(244,63,94,0.1)] font-handwriting text-lg leading-snug tracking-wide'
                }`}>
                  {!isSystem && <p className="whitespace-pre-wrap">{msg.content}</p>}
                  {isSystem && <span className="flex items-center gap-2"><Activity size={10} /> {msg.content}</span>}
                  
                  {/* Heart Particle Burst on Assistant Messages */}
                  {!isSystem && !isUser && <HeartParticleBurst />}

                  {!isSystem && (
                    <div className={`text-[10px] mt-2 opacity-50 flex items-center gap-1 font-sans ${isUser ? 'justify-end text-phoenix-100' : 'text-rose-200/50'}`}>
                      {new Date(msg.timestamp).toLocaleTimeString([], {hour: '2-digit', minute:'2-digit'})}
                    </div>
                  )}

                  {/* Message Actions */}
                  {!isSystem && (
                    <button 
                      onClick={() => deleteMessage(msg.id)}
                      className={`absolute top-2 ${isUser ? '-left-8' : '-right-8'} opacity-0 group-hover:opacity-100 text-gray-500 hover:text-red-400 transition-opacity p-1`}
                      title="Delete Message"
                    >
                      <Trash2 size={14} />
                    </button>
                  )}
                </div>
              </div>
            );
         })}
         <div ref={messagesEndRef} />
       </div>

       <div className="p-4 border-t border-white/5 bg-void-900/80 backdrop-blur-xl relative z-20">
         <div className="relative flex items-center gap-2 max-w-4xl mx-auto">
            <button className="p-3 text-gray-400 hover:text-white hover:bg-white/5 rounded-xl transition-colors">
              <Plus size={20} />
            </button>
           <input
             type="text"
             value={input}
             onChange={(e) => setInput(e.target.value)}
             onKeyDown={(e) => e.key === 'Enter' && handleSend()}
             placeholder={isConnected ? `Message ${currentArchetype?.name || phoenixName}...` : "Connecting to neural interface..."}
             className="w-full bg-void-800/50 border border-white/10 rounded-xl pl-4 pr-12 py-3.5 text-white focus:border-phoenix-500/50 focus:bg-void-800 outline-none transition-all placeholder:text-gray-600"
             disabled={!isConnected}
           />
           <div className="absolute right-2 flex items-center gap-1">
             <button
               onClick={toggleVoiceInput}
               className={`p-2 rounded-lg transition-all ${isListening ? 'text-red-400 bg-red-500/20 animate-pulse' : 'text-gray-400 hover:text-white hover:bg-white/5'}`}
               title="Voice Input"
             >
               <Mic size={18} />
             </button>
             <button 
               onClick={handleSend}
               disabled={!input.trim() || !isConnected}
               className={`p-2 bg-phoenix-600 rounded-lg text-white hover:bg-phoenix-500 disabled:opacity-50 disabled:bg-transparent disabled:text-gray-500 transition-all shadow-lg shadow-phoenix-600/20 ${input.trim() ? 'animate-subtle-bounce' : ''}`}
             >
               <Send size={18} />
             </button>
           </div>
         </div>
       </div>
    </div>
  );
};

// --- Archetype Matcher & Results ---

const MatchResultView = ({ matches, onApply, onRestart, profile }: { matches: Archetype[], onApply: (id: string) => void, onRestart: () => void, profile: DatingProfile }) => {
  const topMatch = matches[0];
  const compatibility = topMatch.matchScore || 0;

  return (
    <div className="animate-in fade-in zoom-in-95 duration-700 h-full flex flex-col items-center justify-center p-6 relative overflow-hidden">
      <div className="absolute top-0 left-0 w-full h-full bg-gradient-to-br from-phoenix-900/20 via-void-900 to-void-900 -z-10" />
      <div className="text-center mb-8">
        <h2 className="text-4xl font-bold mb-2 text-white drop-shadow-lg">It's a Match!</h2>
        <p className="text-phoenix-300 font-medium tracking-wide uppercase text-sm">Compatibility: {compatibility}%</p>
      </div>

      <div className="relative group max-w-sm w-full perspective-1000">
        <div className={`relative bg-gradient-to-br ${topMatch.avatarGradient} p-1 rounded-3xl shadow-[0_0_50px_rgba(0,0,0,0.5)] transform transition-transform duration-500 hover:scale-105`}>
          <div className="bg-void-900/90 backdrop-blur-xl rounded-[22px] p-8 text-center border border-white/10 relative overflow-hidden">
            <h3 className="text-2xl font-bold text-white mb-1">{topMatch.name}</h3>
            <p className="text-sm text-phoenix-400 font-medium uppercase tracking-widest mb-4">{topMatch.sign}</p>
            <p className="text-sm text-gray-300 leading-relaxed mb-6 border-t border-white/10 pt-4">
              "{topMatch.tagline}"
            </p>
            <button 
              onClick={() => onApply(topMatch.id)}
              className="w-full bg-gradient-to-r from-phoenix-600 to-purple-600 hover:from-phoenix-500 hover:to-purple-500 text-white font-bold py-4 rounded-xl shadow-lg transition-all flex items-center justify-center gap-2"
            >
              <Heart className="fill-white" size={20} /> Start Relationship
            </button>
          </div>
        </div>
      </div>
      <button onClick={onRestart} className="mt-8 text-sm text-gray-500 hover:text-gray-300 underline underline-offset-4">
        Start Over
      </button>
    </div>
  );
};

const DatingProfileMatcher = () => {
  const { applyArchetype } = useContext(PhoenixContext)!;
  const [step, setStep] = useState(1);
  const [isMatching, setIsMatching] = useState(false);
  const [matches, setMatches] = useState<Archetype[] | null>(null);
  const totalSteps = 5;
  const [profile, setProfile] = useState<DatingProfile>({
    personalInfo: { name: '', ageRange: '', location: '' },
    communicationStyle: { style: 'Playful', energyLevel: 50, openness: 50, assertiveness: 50, playfulness: 50 },
    emotionalNeeds: { affectionNeed: 50, reassuranceNeed: 50, emotionalAvailability: 50, intimacyDepth: 50, conflictTolerance: 50, impulsivity: 30 },
    loveLanguages: { wordsOfAffirmation: 50, qualityTime: 50, physicalTouch: 50, actsOfService: 50, gifts: 20 },
    attachmentStyle: { style: 'Secure', description: 'Comfortable with intimacy and independence.' },
    relationshipGoals: { goals: [], intimacyComfort: 'Deep' },
    interests: { hobbies: [], favoriteTopics: [] }
  });

  const handleNext = async () => {
    if (step < totalSteps) {
      setStep(s => s + 1);
    } else {
      setIsMatching(true);
      const results = await phoenixService.matchArchetype(profile);
      setMatches(results);
      setIsMatching(false);
    }
  };

  const handleApply = async (archetypeId: string) => {
    await applyArchetype(archetypeId, profile);
  };

  if (matches) return <MatchResultView matches={matches} onApply={handleApply} onRestart={() => { setMatches(null); setStep(1); }} profile={profile} />;
  
  if (isMatching) return (
    <div className="flex flex-col items-center justify-center h-full text-center p-8">
      <Heart size={48} className="text-phoenix-500 animate-pulse mb-4" />
      <h3 className="text-2xl font-bold text-white mb-2">Analyzing Compatibility</h3>
    </div>
  );

  return (
    <div className="max-w-3xl mx-auto h-full flex flex-col p-6">
      <StepIndicator current={step} total={totalSteps} />
      <div className="flex-1 overflow-y-auto custom-scrollbar px-4 pb-4">
        <div className="glass-panel p-8 rounded-2xl min-h-[400px]">
          {step === 1 && (
             <div className="space-y-6 animate-in fade-in slide-in-from-right-4 duration-300">
               <h3 className="text-xl font-semibold text-white">Identity & Basics</h3>
               <div className="space-y-4">
                 <input 
                   type="text" value={profile.personalInfo.name} onChange={e => setProfile({...profile, personalInfo: {...profile.personalInfo, name: e.target.value}})}
                   className="w-full bg-void-900 border border-white/10 rounded-xl px-4 py-3 text-white focus:border-phoenix-500 outline-none" placeholder="What should we call you?"
                 />
                 <input 
                   type="text" value={profile.personalInfo.location} onChange={e => setProfile({...profile, personalInfo: {...profile.personalInfo, location: e.target.value}})}
                   className="w-full bg-void-900 border border-white/10 rounded-xl px-4 py-3 text-white focus:border-phoenix-500 outline-none" placeholder="Location (City, Country)"
                 />
               </div>
               <div className="pt-4">
                  <label className="text-sm text-gray-400 mb-3 block">Communication Style</label>
                  <div className="grid grid-cols-2 gap-3">
                      {['Direct', 'Playful', 'Thoughtful', 'Warm'].map((s: any) => (
                        <SelectionCard 
                          key={s} title={s} selected={profile.communicationStyle.style === s} 
                          onClick={() => setProfile({...profile, communicationStyle: {...profile.communicationStyle, style: s}})}
                          desc={s === 'Direct' ? 'Straight to the point.' : s === 'Playful' ? 'Fun and lighthearted.' : s === 'Thoughtful' ? 'Deep and analytical.' : 'Empathetic and kind.'}
                        />
                      ))}
                  </div>
               </div>
             </div>
          )}
          
          {step === 2 && (
             <div className="space-y-6 animate-in fade-in slide-in-from-right-4 duration-300">
               <h3 className="text-xl font-semibold text-white">Personality Vibe Check</h3>
               <RangeSlider label="Social Energy" value={profile.communicationStyle.energyLevel} onChange={(v:number) => setProfile({...profile, communicationStyle: {...profile.communicationStyle, energyLevel: v}})} minLabel="Introverted" maxLabel="Extroverted" icon={Zap} />
               <RangeSlider label="Playfulness" value={profile.communicationStyle.playfulness} onChange={(v:number) => setProfile({...profile, communicationStyle: {...profile.communicationStyle, playfulness: v}})} minLabel="Serious" maxLabel="Goofy" icon={Smile} />
               <RangeSlider label="Assertiveness" value={profile.communicationStyle.assertiveness} onChange={(v:number) => setProfile({...profile, communicationStyle: {...profile.communicationStyle, assertiveness: v}})} minLabel="Passive" maxLabel="Bold" icon={Shield} />
               <RangeSlider label="Openness" value={profile.communicationStyle.openness} onChange={(v:number) => setProfile({...profile, communicationStyle: {...profile.communicationStyle, openness: v}})} minLabel="Private" maxLabel="Open Book" icon={BookOpen} />
             </div>
          )}

          {step === 3 && (
            <div className="space-y-6 animate-in fade-in slide-in-from-right-4 duration-300">
              <h3 className="text-xl font-semibold text-white">Emotional Profile</h3>
              <div>
                <label className="text-sm text-gray-400 mb-3 block">Attachment Style</label>
                <div className="grid grid-cols-2 gap-3 mb-6">
                  {['Secure', 'Anxious', 'Avoidant', 'Disorganized'].map((s: any) => (
                    <SelectionCard 
                      key={s} title={s} selected={profile.attachmentStyle.style === s} 
                      onClick={() => setProfile({...profile, attachmentStyle: {...profile.attachmentStyle, style: s}})}
                      desc={s === 'Secure' ? 'Comfortable with intimacy.' : s === 'Anxious' ? 'Craves reassurance.' : s === 'Avoidant' ? 'Values independence.' : 'Mixed feelings.'}
                    />
                  ))}
                </div>
              </div>
              <RangeSlider label="Need for Affection" value={profile.emotionalNeeds.affectionNeed} onChange={(v:number) => setProfile({...profile, emotionalNeeds: {...profile.emotionalNeeds, affectionNeed: v}})} minLabel="Independent" maxLabel="Cuddly" icon={Heart} />
              <RangeSlider label="Need for Reassurance" value={profile.emotionalNeeds.reassuranceNeed} onChange={(v:number) => setProfile({...profile, emotionalNeeds: {...profile.emotionalNeeds, reassuranceNeed: v}})} minLabel="Confident" maxLabel="Validated" icon={ShieldCheck} />
            </div>
          )}

          {step === 4 && (
             <div className="space-y-6 animate-in fade-in slide-in-from-right-4 duration-300">
               <h3 className="text-xl font-semibold text-white">Love Languages</h3>
               <p className="text-sm text-gray-400 mb-4">How do you prefer to receive love?</p>
               <RangeSlider label="Words of Affirmation" value={profile.loveLanguages.wordsOfAffirmation} onChange={(v:number) => setProfile({...profile, loveLanguages: {...profile.loveLanguages, wordsOfAffirmation: v}})} minLabel="Low" maxLabel="High" icon={MessageSquare} />
               <RangeSlider label="Quality Time" value={profile.loveLanguages.qualityTime} onChange={(v:number) => setProfile({...profile, loveLanguages: {...profile.loveLanguages, qualityTime: v}})} minLabel="Low" maxLabel="High" icon={Clock} />
               <RangeSlider label="Physical Touch" value={profile.loveLanguages.physicalTouch} onChange={(v:number) => setProfile({...profile, loveLanguages: {...profile.loveLanguages, physicalTouch: v}})} minLabel="Low" maxLabel="High" icon={Hand} />
               <RangeSlider label="Acts of Service" value={profile.loveLanguages.actsOfService} onChange={(v:number) => setProfile({...profile, loveLanguages: {...profile.loveLanguages, actsOfService: v}})} minLabel="Low" maxLabel="High" icon={Briefcase} />
               <RangeSlider label="Gifts" value={profile.loveLanguages.gifts} onChange={(v:number) => setProfile({...profile, loveLanguages: {...profile.loveLanguages, gifts: v}})} minLabel="Low" maxLabel="High" icon={Gift} />
             </div>
          )}

          {step === 5 && (
             <div className="space-y-6 animate-in fade-in slide-in-from-right-4 duration-300">
               <h3 className="text-xl font-semibold text-white">Interests & Goals</h3>
               <div>
                  <label className="text-sm text-gray-400 mb-3 block">Desired Intimacy Level</label>
                  <div className="grid grid-cols-3 gap-3">
                    {['Light', 'Deep', 'Eternal'].map((s: any) => (
                      <SelectionCard 
                        key={s} title={s} selected={profile.relationshipGoals.intimacyComfort === s} 
                        onClick={() => setProfile({...profile, relationshipGoals: {...profile.relationshipGoals, intimacyComfort: s}})}
                        desc={s === 'Light' ? 'Casual fun.' : s === 'Deep' ? 'Serious connection.' : 'Soul merging.'}
                      />
                    ))}
                  </div>
               </div>
               <div className="space-y-4">
                  <div>
                    <label className="text-sm text-gray-400 mb-2 block">Hobbies & Interests</label>
                    <textarea 
                      className="w-full bg-void-900 border border-white/10 rounded-xl p-3 text-white focus:border-phoenix-500 outline-none h-24 resize-none text-sm"
                      placeholder="e.g. Hiking, Coding, Sci-Fi Movies..."
                      value={profile.interests.hobbies.join(', ')}
                      onChange={(e) => setProfile({...profile, interests: {...profile.interests, hobbies: e.target.value.split(', ')}})}
                    />
                  </div>
                  <div>
                    <label className="text-sm text-gray-400 mb-2 block">Favorite Topics</label>
                    <textarea 
                      className="w-full bg-void-900 border border-white/10 rounded-xl p-3 text-white focus:border-phoenix-500 outline-none h-24 resize-none text-sm"
                      placeholder="e.g. Technology, Philosophy, Art..."
                      value={profile.interests.favoriteTopics.join(', ')}
                      onChange={(e) => setProfile({...profile, interests: {...profile.interests, favoriteTopics: e.target.value.split(', ')}})}
                    />
                  </div>
               </div>
             </div>
          )}

        </div>
      </div>
      <div className="flex justify-end mt-6">
        <button onClick={handleNext} className="bg-phoenix-600 text-white px-8 py-3 rounded-xl font-bold flex items-center gap-2 shadow-lg shadow-phoenix-600/20 hover:bg-phoenix-500 transition-all">
          {step === totalSteps ? 'Find Match' : 'Next'} <ArrowRight size={18} />
        </button>
      </div>
    </div>
  );
};

// --- Orchestrator Components ---

const AgentCard: React.FC<{ agent: Agent; onClick: () => void }> = ({ agent, onClick }) => (
  <div onClick={onClick} className="relative glass-panel p-5 rounded-xl border border-white/5 hover:border-phoenix-500 hover:scale-[1.03] hover:shadow-[0_0_20px_rgba(236,72,153,0.15)] cursor-pointer transition-all duration-300 group bg-void-900/50">
    <div className="flex justify-between items-start mb-4">
      <div className={`w-10 h-10 rounded-lg flex items-center justify-center ${agent.status === 'active' ? 'bg-green-500/10 text-green-500' : 'bg-gray-500/10 text-gray-500'}`}>
        <Activity size={20} />
      </div>
      <div className={`px-2 py-1 rounded text-xs font-bold uppercase ${agent.status === 'active' ? 'bg-green-500/10 text-green-500' : 'bg-gray-500/10 text-gray-500'}`}>
        {agent.status}
      </div>
    </div>
    <h3 className="text-white font-bold text-lg mb-1 group-hover:text-phoenix-400 transition-colors">{agent.name}</h3>
    <p className="text-gray-500 text-xs mb-4">{agent.role}</p>

    {/* Tools Display */}
    {agent.tools.length > 0 && (
      <div className="flex flex-wrap gap-1.5 mb-4">
        {agent.tools.map(tId => {
           const toolDef = AVAILABLE_TOOLS.find(t => t.id === tId);
           if (!toolDef) return null;
           const Icon = toolDef.icon;
           return (
             <div key={tId} className="p-1.5 rounded-md bg-white/5 border border-white/5 text-gray-400 hover:text-white hover:bg-white/10 transition-colors" title={toolDef.label}>
               <Icon size={12} />
             </div>
           );
        })}
      </div>
    )}

    <div className="flex items-center gap-2 text-xs text-gray-400">
      <Clock size={12} />
      <span>{agent.uptime}</span>
    </div>
    
    {/* Tooltip for Current Task */}
    {agent.currentTask && (
        <div className="absolute -top-16 left-1/2 -translate-x-1/2 px-3 py-2 bg-black/90 border border-white/10 text-xs text-gray-300 rounded-lg opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all duration-200 pointer-events-none z-20 shadow-xl backdrop-blur-md w-max max-w-[200px] text-center">
            <span className="text-phoenix-400 font-bold block text-[10px] uppercase mb-0.5">Current Task</span>
            {agent.currentTask}
            <div className="absolute -bottom-1 left-1/2 -translate-x-1/2 w-2 h-2 bg-black/90 border-r border-b border-white/10 transform rotate-45"></div>
        </div>
    )}
  </div>
);

const CreateAgentModal = ({ isOpen, onClose, onCreate }: { isOpen: boolean; onClose: () => void; onCreate: (data: Partial<Agent>) => void }) => {
  if (!isOpen) return null;
  return (
    <div className="fixed inset-0 bg-black/80 backdrop-blur-sm z-50 flex items-center justify-center p-4">
      <div className="bg-[#1a1625] border border-white/10 p-6 rounded-xl w-full max-w-md shadow-2xl">
        <h2 className="text-xl font-bold text-white mb-4 flex items-center gap-2">
          <Cpu size={20} className="text-phoenix-500" /> Deploy New Agent
        </h2>
        <div className="space-y-4">
          <input id="agent-name" className="w-full bg-black/50 border border-white/10 rounded p-2 text-white focus:border-phoenix-500 outline-none" placeholder="Agent Name" />
          <input id="agent-role" className="w-full bg-black/50 border border-white/10 rounded p-2 text-white focus:border-phoenix-500 outline-none" placeholder="Role" />
          <textarea id="agent-mission" className="w-full bg-black/50 border border-white/10 rounded p-2 text-white focus:border-phoenix-500 outline-none h-24 resize-none" placeholder="Mission..." />
          <div className="flex justify-end gap-3 mt-6">
            <button onClick={onClose} className="text-gray-400 text-sm">Cancel</button>
            <button onClick={() => {
              const name = (document.getElementById('agent-name') as HTMLInputElement).value;
              const role = (document.getElementById('agent-role') as HTMLInputElement).value;
              const mission = (document.getElementById('agent-mission') as HTMLInputElement).value;
              onCreate({ name, role, mission });
              onClose();
            }} className="px-4 py-2 bg-phoenix-500 text-white rounded text-sm font-bold">Deploy</button>
          </div>
        </div>
      </div>
    </div>
  );
};

const OrchestratorView = () => {
  const [agents, setAgents] = useState<Agent[]>(MOCK_AGENTS);
  const [selectedAgentId, setSelectedAgentId] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<'overview' | 'tools' | 'logs'>('overview');
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);
  const [editedMission, setEditedMission] = useState('');

  const selectedAgent = agents.find(a => a.id === selectedAgentId);

  useEffect(() => {
    if (selectedAgent) {
      setEditedMission(selectedAgent.mission);
    }
  }, [selectedAgentId, selectedAgent]);

  const toggleTool = (tool: string) => {
    if (!selectedAgent) return;
    const hasTool = selectedAgent.tools.includes(tool);
    const updatedAgents = agents.map(a => 
      a.id === selectedAgent.id 
        ? { ...a, tools: hasTool ? a.tools.filter(t => t !== tool) : [...a.tools, tool] } 
        : a
    );
    setAgents(updatedAgents);
  };

  const handleCreateAgent = (data: Partial<Agent>) => {
    const newAgent: Agent = {
      id: `agent_${Date.now()}`,
      name: data.name || 'New Agent',
      role: data.role || 'Generalist',
      status: 'idle',
      mission: data.mission || 'Awaiting instructions...',
      tools: [],
      currentTask: null,
      uptime: '0s',
      logs: ['[System] Node initialized.']
    };
    setAgents([...agents, newAgent]);
    setSelectedAgentId(newAgent.id);
  };

  const handleUpdateMission = () => {
    if (!selectedAgent) return;
    const updatedAgents = agents.map(a => 
      a.id === selectedAgent.id ? { ...a, mission: editedMission } : a
    );
    setAgents(updatedAgents);
  };

  if (selectedAgent) {
    return (
      <div className="flex flex-col h-full bg-[#0f0b15]">
        <div className="h-16 border-b border-white/5 flex items-center justify-between px-6 bg-void-800/50 backdrop-blur-md">
          <div className="flex items-center gap-4">
             <button onClick={() => setSelectedAgentId(null)} className="text-gray-400 hover:text-white transition-colors">
               <ChevronRight size={20} className="rotate-180" />
             </button>
             <div className="flex flex-col">
               <h2 className="text-lg font-bold text-white flex items-center gap-2">
                 {selectedAgent.name}
                 <span className={`w-2 h-2 rounded-full ${selectedAgent.status === 'active' ? 'bg-green-500' : 'bg-gray-500'}`} />
               </h2>
               <span className="text-xs text-gray-500 font-mono">{selectedAgent.id}</span>
             </div>
          </div>
        </div>

        <div className="flex border-b border-white/5 px-6">
           {['overview', 'tools', 'logs'].map(tab => (
             <button 
               key={tab}
               onClick={() => setActiveTab(tab as any)}
               className={`px-4 py-3 text-sm font-medium border-b-2 capitalize transition-colors ${activeTab === tab ? 'border-phoenix-500 text-white' : 'border-transparent text-gray-500 hover:text-gray-300'}`}
             >
               {tab}
             </button>
           ))}
        </div>

        <div className="flex-1 overflow-y-auto p-8 max-w-5xl mx-auto w-full">
           {activeTab === 'overview' && (
             <div className="space-y-6">
               <div className="glass-panel p-6 rounded-xl">
                 <h3 className="text-sm font-bold text-gray-400 uppercase tracking-wider mb-4">Mission Directive</h3>
                 <textarea 
                   className="w-full h-32 bg-void-900 border border-white/10 rounded-lg p-4 text-gray-200 focus:border-phoenix-500 outline-none resize-none leading-relaxed"
                   value={editedMission}
                   onChange={(e) => setEditedMission(e.target.value)}
                 />
                 <div className="mt-2 text-right">
                   <button onClick={handleUpdateMission} className="text-xs text-phoenix-400 hover:text-phoenix-300 font-semibold">Update Directive</button>
                 </div>
               </div>
             </div>
           )}
           {activeTab === 'tools' && (
             <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
               {AVAILABLE_TOOLS.map(tool => {
                 const isActive = selectedAgent.tools.includes(tool.id);
                 return (
                   <div 
                     key={tool.id} 
                     onClick={() => toggleTool(tool.id)} 
                     className={`relative group p-4 rounded-xl border cursor-pointer transition-all ${isActive ? 'bg-phoenix-900/20 border-phoenix-500/50' : 'bg-void-900/50 border-white/5 hover:bg-void-800'}`}
                   >
                     <div className="flex items-center gap-3">
                       <tool.icon size={20} className={isActive ? 'text-phoenix-500' : 'text-gray-500'} />
                       <span className={isActive ? 'text-white' : 'text-gray-400'}>{tool.label}</span>
                     </div>
                     {/* Tooltip */}
                     <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-3 px-3 py-2 bg-black/90 border border-white/10 text-xs text-gray-300 rounded-lg opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all duration-200 pointer-events-none z-20 shadow-xl backdrop-blur-md w-48 text-center">
                        {tool.desc}
                        <div className="absolute -bottom-1 left-1/2 -translate-x-1/2 w-2 h-2 bg-black/90 border-r border-b border-white/10 transform rotate-45"></div>
                     </div>
                   </div>
                 );
               })}
             </div>
           )}
           {activeTab === 'logs' && (
             <div className="bg-black/50 rounded-xl border border-white/10 p-4 font-mono text-sm h-[400px] overflow-y-auto custom-scrollbar">
               {selectedAgent.logs.map((log, i) => <div key={i} className="mb-2 text-gray-300 border-b border-white/5 pb-1"><span className="text-phoenix-500 mr-2">$</span>{log}</div>)}
             </div>
           )}
        </div>
      </div>
    );
  }

  return (
    <div className="p-8 h-full overflow-y-auto custom-scrollbar flex flex-col">
      <CreateAgentModal isOpen={isCreateModalOpen} onClose={() => setIsCreateModalOpen(false)} onCreate={handleCreateAgent} />
      <div className="flex justify-between items-end mb-8">
        <div>
          <h2 className="text-3xl font-bold mb-2 gradient-text">Neural Orchestration Layer</h2>
          <p className="text-gray-400">Manage autonomous sub-agents, assign tools, and monitor mission status.</p>
        </div>
        <button onClick={() => setIsCreateModalOpen(true)} className="bg-white text-black px-4 py-2 rounded-lg font-semibold flex items-center gap-2 text-sm"><Plus size={16} /> Deploy New Agent</button>
      </div>
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {agents.map(agent => <AgentCard key={agent.id} agent={agent} onClick={() => setSelectedAgentId(agent.id)} />)}
      </div>
    </div>
  );
};

// --- Layout & App ---

const SidebarItem = ({ icon: Icon, label, active, onClick, danger }: any) => (
  <button
    onClick={onClick}
    className={`w-full flex items-center gap-3 px-4 py-3 rounded-xl transition-all duration-200 group ${
      active 
        ? 'bg-phoenix-600/10 text-phoenix-400 border border-phoenix-500/20 shadow-[0_0_15px_rgba(236,72,153,0.1)]' 
        : danger 
          ? 'text-red-400 hover:bg-red-500/10 hover:text-red-300' 
          : 'text-gray-400 hover:text-white hover:bg-white/5'
    }`}
  >
    <Icon size={18} className={`transition-transform duration-300 ${active ? 'scale-110' : 'group-hover:scale-110'}`} />
    <span className="text-sm font-medium">{label}</span>
    {active && <div className="ml-auto w-1.5 h-1.5 rounded-full bg-phoenix-500 shadow-[0_0_8px_rgba(236,72,153,0.8)]" />}
  </button>
);

const DashboardLayout = () => {
  const { clearHistory, relationalScore, sentiment, setRelationalScore, setSentiment, isConnected, phoenixName } = useContext(PhoenixContext)!;
  const [activeView, setActiveView] = useState<'chat' | 'archetype' | 'settings' | 'memories' | 'orchestrator' | 'studio' | 'google' | 'devtools'>('chat');
  const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false);
  const [isClearModalOpen, setIsClearModalOpen] = useState(false);

  const handleNavigation = (view: typeof activeView) => {
    setActiveView(view);
    setIsMobileMenuOpen(false);
  };

  return (
    <div className="flex h-screen w-full bg-[#0f0b15] overflow-hidden font-sans">
      <ConfirmationModal 
        isOpen={isClearModalOpen}
        onClose={() => setIsClearModalOpen(false)}
        onConfirm={clearHistory}
        title="Wipe Memory Banks?"
        message="This will permanently delete the current conversation history."
      />

      <div className={`fixed inset-y-0 left-0 z-50 w-72 bg-void-800 border-r border-white/5 transform transition-transform duration-300 lg:relative lg:translate-x-0 ${isMobileMenuOpen ? 'translate-x-0' : '-translate-x-full'}`}>
        <div className="p-6 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <DynamicHeartLogo score={relationalScore} sentiment={sentiment} isConnected={isConnected} size={36} />
            <h1 className="text-xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-white to-gray-400">{phoenixName.toUpperCase()}</h1>
          </div>
          <button onClick={() => setIsMobileMenuOpen(false)} className="lg:hidden text-gray-400"><X size={24} /></button>
        </div>

        <div className="px-4 space-y-2 mt-4">
          <div className="text-xs font-semibold text-gray-500 uppercase tracking-wider px-4 mb-2">Dashboard</div>
          <SidebarItem icon={MessageSquare} label="Chat Stream" active={activeView === 'chat'} onClick={() => handleNavigation('chat')} />
          <SidebarItem icon={Film} label="Studio & Recording" active={activeView === 'studio'} onClick={() => handleNavigation('studio')} />
          <SidebarItem icon={Network} label="Orchestrator" active={activeView === 'orchestrator'} onClick={() => handleNavigation('orchestrator')} />
          <SidebarItem icon={Cloud} label="Google Ecosystem" active={activeView === 'google'} onClick={() => handleNavigation('google')} />
          <SidebarItem icon={Heart} label="Archetype Matcher" active={activeView === 'archetype'} onClick={() => handleNavigation('archetype')} />
          <SidebarItem icon={Brain} label="Memories & Context" active={activeView === 'memories'} onClick={() => handleNavigation('memories')} />
        </div>

        <div className="px-4 space-y-2 mt-8">
          <div className="text-xs font-semibold text-gray-500 uppercase tracking-wider px-4 mb-2">System</div>
          <SidebarItem icon={Trash2} label="Clear Memory" active={false} danger onClick={() => { setIsClearModalOpen(true); setIsMobileMenuOpen(false); }} />
          <SidebarItem icon={Terminal} label="Self-Mod Console" active={activeView === 'devtools'} onClick={() => handleNavigation('devtools')} />
          <SidebarItem icon={Settings} label="Settings" active={activeView === 'settings'} onClick={() => handleNavigation('settings')} />
        </div>
      </div>

      <div className="flex-1 flex flex-col h-full overflow-hidden relative">
        <div className="lg:hidden h-16 flex items-center px-4 border-b border-white/5 justify-between">
          <button onClick={() => setIsMobileMenuOpen(true)} className="text-gray-300"><Menu size={24} /></button>
          <span className="font-semibold text-gray-200 capitalize">{activeView}</span>
          <div className="w-6" />
        </div>

        <div className="flex-1 overflow-hidden relative bg-gradient-to-b from-[#0f0b15] to-[#130f1c]">
          {activeView === 'chat' && <ChatView />}
          {activeView === 'archetype' && <DatingProfileMatcher />}
          {activeView === 'orchestrator' && <OrchestratorView />}
          {activeView === 'studio' && <StudioView />}
          {activeView === 'google' && <GoogleEcosystemView />}
          {activeView === 'devtools' && <DevToolsView />}
          {activeView === 'memories' && (
            <div className="flex flex-col items-center justify-center h-full text-center p-8">
              <Brain size={64} className="text-gray-700 mb-4" />
              <h3 className="text-xl font-bold text-gray-300">Memory Banks Locked</h3>
              <p className="text-gray-500 max-w-md mt-2">Vector database not connected.</p>
            </div>
          )}
          {activeView === 'settings' && (
             <div className="p-8 max-w-2xl mx-auto h-full flex flex-col">
               <h2 className="text-2xl font-bold mb-6">System Configuration</h2>
               <div className="glass-panel p-6 rounded-xl mb-6">
                 <h3 className="text-lg font-medium mb-4">Relational Diagnostics</h3>
                 <input type="range" min="0" max="100" value={relationalScore} onChange={(e) => setRelationalScore(Number(e.target.value))} className="w-full h-2 bg-void-700 rounded-lg accent-phoenix-500" />
               </div>
             </div>
          )}
        </div>
      </div>
    </div>
  );
};

// Mount
const rootElement = document.getElementById('root');
if (rootElement) {
  const root = createRoot(rootElement);
  root.render(
    <PhoenixProvider>
      <DashboardLayout />
    </PhoenixProvider>
  );
}
