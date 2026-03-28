<script lang="ts">
  let { stack, size = 14 }: { stack: string; size?: number } = $props();

  // Colors per stack
  const COLORS: Record<string, string> = {
    rust:    '#ce422b',
    node:    '#339933',
    go:      '#00acd7',
    python:  '#3572a5',
    java:    '#b07219',
    flutter: '#54c5f8',
    dotnet:  '#512bd4',
    php:     '#4f5d95',
    ruby:    '#cc342d',
    cpp:     '#f34b7d',
    make:    '#427819',
    swift:   '#f05138',
    git:     '#f1502f',
  };

  const LABELS: Record<string, string> = {
    rust:    'RS',
    node:    'JS',
    go:      'GO',
    python:  'PY',
    java:    'JV',
    flutter: 'FL',
    dotnet:  'C#',
    php:     'PHP',
    ruby:    'RB',
    cpp:     'C++',
    make:    'MK',
    swift:   'SW',
    git:     'GIT',
  };

  const color = $derived(COLORS[stack] ?? '#8b8fa8');
  const label = $derived(LABELS[stack] ?? stack.slice(0, 2).toUpperCase());
  const fontSize = $derived(Math.max(7, Math.round(size * 0.56)));
  const radius = $derived(Math.round(size * 0.25));
</script>

<span
  class="stack-badge"
  style="
    width: {size * 1.8}px;
    height: {size}px;
    background: {color}22;
    color: {color};
    border-radius: {radius}px;
    font-size: {fontSize}px;
  "
  title={stack}
>{label}</span>

<style>
  .stack-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    font-weight: 700;
    letter-spacing: -0.02em;
    flex-shrink: 0;
    font-family: 'SF Mono', 'JetBrains Mono', monospace;
    white-space: nowrap;
  }
</style>
