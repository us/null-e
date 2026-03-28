/**
 * Technology logos as inline SVG components.
 * Maps real backend ProjectKind values (NodeNpm, PythonPip, etc.)
 * to brand-colored icons.
 */

interface IconProps {
  size?: number;
  className?: string;
}

// ─── Technology Icons ───────────────────────────────────────

function NodeIcon({ size = 24, className }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" className={className}>
      <path d="M12 1.85l9.66 5.58v11.14L12 24.15l-9.66-5.58V7.43L12 1.85z" fill="#8CC84B" />
      <path d="M12 1.85v22.3l9.66-5.58V7.43L12 1.85z" fill="#6BA544" />
    </svg>
  );
}

function RustIcon({ size = 24, className }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" className={className}>
      <path d="M23.8 11.4l-1.2-.7.1-.5 1-.9c.1-.2.1-.3-.1-.4l-1.3-.4v-.5l.8-1.1c.1-.1 0-.3-.2-.4l-1.3-.1-.1-.5.5-1.2c.1-.2 0-.3-.2-.3l-1.3.2-.2-.5.2-1.3c0-.2-.1-.3-.3-.3l-1.2.5-.3-.4-.1-1.3c0-.2-.2-.3-.3-.2l-1.1.7-.4-.3-.5-1.2c-.1-.2-.2-.2-.4-.1l-.9 1-.5-.2L14.2.3c-.1-.2-.3-.2-.4 0l-.7 1.2-.5-.1-1-.9c-.1-.1-.3-.1-.4.1l-.4 1.3-.5.1-1.1-.8c-.2-.1-.3 0-.4.2l-.1 1.3-.5.2L7 2c-.2-.1-.3 0-.3.2l.2 1.3-.5.2-1.2-.5c-.2-.1-.3 0-.3.2l.5 1.2-.4.3-1.3.1c-.2 0-.3.2-.2.3l.7 1.1-.3.4-1.2.5c-.2.1-.2.2-.1.4l1 .9-.2.5-1.3.4c-.2.1-.2.2-.1.4l1.1.8v.5l-1.3.4c-.2.1-.2.2 0 .4l1.2.7v.5l-1.2.7c-.2.1-.2.3 0 .4l1.3.4v.5l-1 .8c-.1.2-.1.3.1.4l1.3.4v.5l-.8 1.1c-.1.2 0 .3.2.4l1.3.1.1.5-.5 1.2c-.1.2 0 .3.2.3l1.3-.2.2.5-.2 1.3c0 .2.1.3.3.3l1.2-.5.3.4.1 1.3c0 .2.2.3.3.2l1.1-.7.4.3.5 1.2c.1.2.2.2.4.1l.9-1 .5.2.4 1.3c.1.2.3.2.4 0l.7-1.2.5.1 1 .9c.2.1.3.1.4-.1l.4-1.3.5-.1 1.1.8c.2.1.3 0 .4-.2l.1-1.3.5-.2 1.2.5c.2.1.3 0 .3-.2l-.2-1.3.5-.2 1.2.5c.2.1.3 0 .3-.2l-.5-1.2.4-.3 1.3-.1c.2 0 .3-.2.2-.3l-.7-1.1.3-.4 1.2-.5c.2-.1.2-.2.1-.4l-1-.9.2-.5 1.3-.4c.2-.1.2-.2.1-.4l-1.1-.8v-.5l1.3-.4c.2-.1.2-.2 0-.4l-1.2-.7zM12 18a6 6 0 110-12 6 6 0 010 12z" fill="#F74C00" />
    </svg>
  );
}

function PythonIcon({ size = 24, className }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" className={className}>
      <path d="M11.9 1C6.4 1 7 3.5 7 3.5v2.6h5v.8H4.5S1 6.4 1 12s2.8 5.4 2.8 5.4H6v-2.6s-.1-2.8 2.8-2.8h4.8s2.7 0 2.7-2.6V4.2S16.8 1 11.9 1zm-2.7 1.9a.9.9 0 110 1.8.9.9 0 010-1.8z" fill="#3776AB" />
      <path d="M12.1 23c5.5 0 4.9-2.5 4.9-2.5v-2.6h-5v-.8h7.5S23 17.6 23 12s-2.8-5.4-2.8-5.4H18v2.6s.1 2.8-2.8 2.8H10.4s-2.7 0-2.7 2.6v5.2S7.2 23 12.1 23zm2.7-1.9a.9.9 0 110-1.8.9.9 0 010 1.8z" fill="#FFD43B" />
    </svg>
  );
}

function GoIcon({ size = 24, className }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" className={className}>
      <path d="M1.8 9.7s-.1-.1 0-.2h.1l.1.1 1.7 1.2h.1l-.2.2-1.7-1.3zm20.4 0l-1.8-1.2h-.1l-.2.2 1.7 1.2h.1s.1 0 .1-.1l.2-.1z" fill="#00ACD7" />
      <path d="M19.3 10.1c-.1-.3-.2-.7-.3-1-.4-1-1.1-1.8-2.1-2.4-.6-.4-1.3-.6-2-.7h-1.3c-.5.1-1 .2-1.5.4-1 .4-1.8 1-2.4 1.9-.5.7-.7 1.5-.8 2.4v1c.1.8.3 1.5.7 2.2.5.9 1.3 1.6 2.2 2 .7.3 1.4.5 2.1.5h1.2c.5-.1 1-.2 1.5-.4 1-.4 1.7-1 2.3-1.9.5-.7.7-1.5.8-2.3v-1c0-.3-.1-.5-.2-.7z" fill="#00ACD7" />
      <ellipse cx="8.6" cy="10.7" rx="1" ry=".9" fill="#00ACD7" />
      <ellipse cx="15.4" cy="10.7" rx="1" ry=".9" fill="#00ACD7" />
    </svg>
  );
}

function JavaIcon({ size = 24, className }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" className={className}>
      <path d="M8.9 18.6s-1 .6.7.8c2 .2 3.1.2 5.3-.2 0 0 .6.4 1.4.7-5 2.1-11.3-.1-7.4-1.3zm-.6-3s-1.1.8.6.9c2.2.2 3.9.2 6.9-.3 0 0 .4.4 1 .6-6 1.8-12.7.1-8.5-1.2z" fill="#ED8B00" />
      <path d="M13.5 10.7c1.2 1.4-.3 2.6-.3 2.6s3.1-1.6 1.7-3.6c-1.3-1.9-2.4-2.8 3.2-6.1 0 0-8.7 2.2-4.6 7.1z" fill="#ED8B00" />
      <path d="M18.7 20.2s.7.6-.8 1.1c-2.9.9-12 1.2-14.5 0-.9-.4.8-1 1.3-1.1.5-.1.8-.1.8-.1-1-.7-6.1 1.3-2.6 1.9 9.4 1.6 17.2-.7 15.8-1.8z" fill="#ED8B00" />
      <path d="M9.3 13.3s-4.3 1-1.5 1.4c1.2.2 3.5.1 5.7-.1 1.8-.2 3.5-.5 3.5-.5s-.6.3-1.1.5c-4.2 1.1-12.3.6-10--.5 2-1 3.4-.8 3.4-.8zM16.5 16s.9.7-.9 1.2c-3.5 1-14.7.1-11.9-1 1.2-.5 2-.4 2-.4s-.5-.3-1.8-.6c-5.4 1.6 2.9 3.3 12.6 1.4-.2-.1 0-.1 0-.1z" fill="#ED8B00" />
    </svg>
  );
}

function DockerIcon({ size = 24, className }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" className={className}>
      <path d="M13.98 11.08h2.12V8.9h-2.12v2.18zm-2.74 0h2.12V8.9h-2.12v2.18zm-2.74 0h2.12V8.9H8.5v2.18zm-2.74 0h2.12V8.9H5.76v2.18zm2.74-2.8h2.12V6.1H8.5v2.18zm2.74 0h2.12V6.1h-2.12v2.18zm2.74 0h2.12V6.1h-2.12v2.18zm0-2.8h2.12V3.3h-2.12v2.18zM23 11.47c-.6-.36-1.98-.49-3.04-.31-.14-.98-.7-1.83-1.72-2.6l-.59-.39-.39.59c-.5.75-.63 1.98-.5 2.93.1.7.37 1.39.82 1.92-.38.22-.81.4-1.21.53-.7.2-1.46.31-2.2.31H.54l-.05.64c-.1 1.18.07 2.37.5 3.47l.2.42.03.05c1.39 2.31 3.86 3.34 6.62 3.34 5.7 0 10.4-2.53 12.65-8 1.37.07 2.78-.16 3.62-1.27l.44-.62-.55-.41z" fill="#2496ED" />
    </svg>
  );
}

function DotNetIcon({ size = 24, className }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" className={className}>
      <circle cx="12" cy="12" r="11" fill="#512BD4" />
      <text x="12" y="16.5" textAnchor="middle" fontSize="10" fontWeight="bold" fill="white" fontFamily="sans-serif">.N</text>
    </svg>
  );
}

function RubyIcon({ size = 24, className }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" className={className}>
      <path d="M5.1 22l1.7-8.8L1 8.5 8.2 2l7.5.3L22 7.3l-.9 8.4-5.7 6.3H5.1z" fill="#CC342D" />
      <path d="M15.7 2.3L22 7.3l-.9 8.4-5.7 6.3H5.1l1.7-8.8L15.7 2.3z" fill="#A12A23" opacity="0.6" />
    </svg>
  );
}

function SwiftIcon({ size = 24, className }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" className={className}>
      <rect rx="5.5" width="24" height="24" fill="#F05138" />
      <path d="M16.2 17.8c-.2.1-.5.2-.8.2-2.3.4-5-.4-7.1-2.6 0 0 3.6 2.3 6.8.8-2.3-1.4-4.2-3.5-5.5-5.8 1.7 1.5 3.5 2.6 5.1 3.2C12.5 11.5 11 9.5 10 8c1.3 1.2 3.8 3.4 4 3.5-.8-1.5-1.3-3.2-1.3-4.5 1.6 2.3 2.7 4.4 3 5.7.4-1.1.5-2.8.1-5 1.7 2.7 1.3 6 .4 7.5l.1.1c2-1.7 3.2-4.5 2.5-7.3 0 0 .6 3.5-1.1 6.5.3.3.5.5.5.5s1.5-2.1 1.6-5c0 1.9-.6 4.2-2.1 5.6-.5.6-1 1-1.5 1.2z" fill="white" />
    </svg>
  );
}

function DartIcon({ size = 24, className }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" className={className}>
      <path d="M4.1 20L1 7l6.1-6h9.8L23 7l-3.1 13H4.1z" fill="#00B4AB" />
      <path d="M7.1 1l-6.1 6 3.1 13h3l-2-10.5L7.1 1z" fill="#00897B" opacity="0.5" />
    </svg>
  );
}

function FlutterIcon({ size = 24, className }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" className={className}>
      <path d="M14.3 1L3 12.3l3.5 3.5L20.3 1h-6zm0 10.2L8.5 17l5.8 5.8h6l-5.8-5.8 5.8-5.8h-6z" fill="#42A5F5" />
      <path d="M8.5 17l2.9-2.9 2.9 2.9-2.9 2.9z" fill="#0D47A1" />
    </svg>
  );
}

function PhpIcon({ size = 24, className }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" className={className}>
      <ellipse cx="12" cy="12" rx="11" ry="7" fill="#777BB3" />
      <text x="12" y="15" textAnchor="middle" fontSize="8" fontWeight="bold" fill="white" fontFamily="sans-serif">php</text>
    </svg>
  );
}

function ElixirIcon({ size = 24, className }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" className={className}>
      <path d="M12 2c-2 2-6 7-6 12a6 6 0 1012 0c0-5-4-10-6-12z" fill="#6E4A7E" />
      <path d="M12 2c-2 2-6 7-6 12a6 6 0 006 6c0-6 3-12 0-18z" fill="#9B72AA" opacity="0.5" />
    </svg>
  );
}

function CppIcon({ size = 24, className }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" className={className}>
      <circle cx="12" cy="12" r="11" fill="#00599C" />
      <text x="12" y="16" textAnchor="middle" fontSize="9" fontWeight="bold" fill="white" fontFamily="sans-serif">C++</text>
    </svg>
  );
}

function TerraformIcon({ size = 24, className }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" className={className}>
      <path d="M1 4.4v6.6l5.7 3.3V7.7L1 4.4z" fill="#5C4EE5" />
      <path d="M7.5 7.7v6.6l5.7-3.3V4.4L7.5 7.7z" fill="#4040B2" />
      <path d="M7.5 15.3v6.6l5.7-3.3V12L7.5 15.3z" fill="#4040B2" />
      <path d="M14 4.4v6.6l5.7-3.3V1.1L14 4.4z" fill="#5C4EE5" />
    </svg>
  );
}

function ZigIcon({ size = 24, className }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" className={className}>
      <path d="M2 6h7l-4 12H2l4-12zm6 0h8l-4 12H4l4-12zm7 0h7l-4 12h-7l4-12z" fill="#F7A41D" />
    </svg>
  );
}

function DefaultIcon({ size = 24, className }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" className={className}>
      <rect rx="4" width="24" height="24" fill="var(--color-text-muted)" opacity="0.3" />
      <path d="M8 6h8v2H8zm0 4h8v2H8zm0 4h5v2H8z" fill="var(--color-text-muted)" />
    </svg>
  );
}

// ─── Kind → Icon mapping ────────────────────────────────────
// Backend sends ProjectKind debug strings like "NodeNpm", "PythonPip", etc.

const iconMap: Record<string, (props: IconProps) => React.ReactNode> = {
  // Node.js ecosystem
  NodeNpm: NodeIcon,
  NodeYarn: NodeIcon,
  NodePnpm: NodeIcon,
  NodeBun: NodeIcon,
  Deno: NodeIcon,
  // Rust
  Rust: RustIcon,
  // Python ecosystem
  PythonPip: PythonIcon,
  PythonPoetry: PythonIcon,
  PythonPipenv: PythonIcon,
  PythonConda: PythonIcon,
  PythonUv: PythonIcon,
  // Go
  Go: GoIcon,
  // JVM ecosystem
  JavaMaven: JavaIcon,
  JavaGradle: JavaIcon,
  Kotlin: JavaIcon,
  Scala: JavaIcon,
  Clojure: JavaIcon,
  // .NET
  DotNet: DotNetIcon,
  FSharp: DotNetIcon,
  // Docker
  Docker: DockerIcon,
  // Ruby
  RubyBundler: RubyIcon,
  RubyRails: RubyIcon,
  // Swift/Apple
  SwiftSpm: SwiftIcon,
  SwiftXcode: SwiftIcon,
  // Mobile
  Flutter: FlutterIcon,
  ReactNative: NodeIcon,
  Android: JavaIcon,
  // Dart
  Dart: DartIcon,
  // PHP
  PhpComposer: PhpIcon,
  PhpLaravel: PhpIcon,
  // Other
  Elixir: ElixirIcon,
  Cpp: CppIcon,
  C: CppIcon,
  Zig: ZigIcon,
  // IaC
  Terraform: TerraformIcon,
  Pulumi: TerraformIcon,
  // Legacy short names (fallback)
  Node: NodeIcon,
  Python: PythonIcon,
  Java: JavaIcon,
  Ruby: RubyIcon,
  Swift: SwiftIcon,
};

interface TechIconProps extends IconProps {
  kind: string;
}

export function TechIcon({ kind, size = 24, className }: TechIconProps) {
  const Icon = iconMap[kind] ?? DefaultIcon;
  return <Icon size={size} className={className} />;
}

// ─── Kind → display name ────────────────────────────────────

const kindLabels: Record<string, string> = {
  NodeNpm: 'Node.js',
  NodeYarn: 'Node.js',
  NodePnpm: 'Node.js',
  NodeBun: 'Node.js',
  Deno: 'Deno',
  Rust: 'Rust',
  PythonPip: 'Python',
  PythonPoetry: 'Python',
  PythonPipenv: 'Python',
  PythonConda: 'Python',
  PythonUv: 'Python',
  Go: 'Go',
  JavaMaven: 'Java',
  JavaGradle: 'Java',
  Kotlin: 'Kotlin',
  Scala: 'Scala',
  Clojure: 'Clojure',
  DotNet: '.NET',
  FSharp: 'F#',
  Docker: 'Docker',
  RubyBundler: 'Ruby',
  RubyRails: 'Rails',
  SwiftSpm: 'Swift',
  SwiftXcode: 'Xcode',
  Flutter: 'Flutter',
  ReactNative: 'React Native',
  Android: 'Android',
  Dart: 'Dart',
  PhpComposer: 'PHP',
  PhpLaravel: 'Laravel',
  Elixir: 'Elixir',
  Haskell: 'Haskell',
  OCaml: 'OCaml',
  Cpp: 'C++',
  C: 'C',
  Zig: 'Zig',
  Terraform: 'Terraform',
  Pulumi: 'Pulumi',
};

export function getKindLabel(kind: string): string {
  return kindLabels[kind] ?? kind;
}

// ─── Kind → brand color ─────────────────────────────────────

const techColors: Record<string, string> = {
  NodeNpm: '#8CC84B',
  NodeYarn: '#2C8EBB',
  NodePnpm: '#F69220',
  NodeBun: '#FBF0DF',
  Deno: '#12124B',
  Rust: '#F74C00',
  PythonPip: '#3776AB',
  PythonPoetry: '#3776AB',
  PythonPipenv: '#3776AB',
  PythonConda: '#44A833',
  PythonUv: '#DE5FE9',
  Go: '#00ACD7',
  JavaMaven: '#ED8B00',
  JavaGradle: '#02303A',
  Kotlin: '#7F52FF',
  Scala: '#DC322F',
  DotNet: '#512BD4',
  FSharp: '#378BBA',
  Docker: '#2496ED',
  RubyBundler: '#CC342D',
  RubyRails: '#CC0000',
  SwiftSpm: '#F05138',
  SwiftXcode: '#147EFB',
  Flutter: '#42A5F5',
  ReactNative: '#61DAFB',
  Android: '#3DDC84',
  Dart: '#00B4AB',
  PhpComposer: '#777BB3',
  PhpLaravel: '#FF2D20',
  Elixir: '#6E4A7E',
  Cpp: '#00599C',
  C: '#A8B9CC',
  Zig: '#F7A41D',
  Terraform: '#5C4EE5',
  Pulumi: '#8A3391',
  // Legacy short names
  Node: '#8CC84B',
  Python: '#3776AB',
  Java: '#ED8B00',
  Ruby: '#CC342D',
  Swift: '#F05138',
};

export function getTechColor(kind: string): string {
  return techColors[kind] ?? '#64748B';
}

// ─── Artifact kind → icon (emoji fallback) ──────────────────

const artifactIcons: Record<string, string> = {
  Dependencies: '📦',
  BuildOutput: '🔨',
  Cache: '🗄️',
  VirtualEnv: '🐍',
  IdeArtifacts: '⚙️',
  TestOutput: '🧪',
  Logs: '📋',
  Temporary: '🗑️',
  Docker: '🐳',
  PackageManagerCache: '📥',
  Bytecode: '⚡',
  DocsBuild: '📖',
};

export function getArtifactIcon(artifactKind: string): string {
  return artifactIcons[artifactKind] ?? '📁';
}
