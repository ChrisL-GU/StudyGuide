<script lang="ts">
  import type { Snippet } from 'svelte';
  import { page } from '$app/state';

  let { children }: { children: Snippet } = $props();

  let collapsed = $state(false);

  const navItems = [
    { href: '/documents', label: 'Documents', icon: 'M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z' },
    { href: '/chat', label: 'Chat', icon: 'M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z' },
    { href: '/settings', label: 'Settings', icon: 'M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 11-6 0 3 3 0 016 0z' },
  ];
</script>

<div class="app-layout">
  <nav class="sidebar" class:collapsed>
    <div class="sidebar-brand">
      <svg class="brand-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" />
      </svg>
      {#if !collapsed}<span>StudyGuide</span>{/if}
    </div>

    <div class="nav-links">
      {#each navItems as item}
        <a href={item.href} class="nav-link" class:active={page.url.pathname.startsWith(item.href)} title={item.label}>
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
            <path d={item.icon} />
          </svg>
          {#if !collapsed}<span>{item.label}</span>{/if}
        </a>
      {/each}
    </div>

    <div class="sidebar-footer">
      <button class="collapse-btn" onclick={() => collapsed = !collapsed} title={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class:flipped={collapsed}>
          <path d="M15 19l-7-7 7-7" />
        </svg>
      </button>
    </div>
  </nav>
  <main class="main-content">
    <div class="content-inner">
      {@render children()}
    </div>
  </main>
</div>

<style>
  :global(body) {
    margin: 0;
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    font-size: 14px;
    line-height: 1.6;
    color: #e2e8f0;
    background-color: #0c0f1a;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
  }

  :global(*) {
    box-sizing: border-box;
  }

  :global(::selection) {
    background: rgba(99, 102, 241, 0.3);
  }

  :global(::-webkit-scrollbar) {
    width: 6px;
  }

  :global(::-webkit-scrollbar-track) {
    background: transparent;
  }

  :global(::-webkit-scrollbar-thumb) {
    background: rgba(148, 163, 184, 0.15);
    border-radius: 3px;
  }

  :global(::-webkit-scrollbar-thumb:hover) {
    background: rgba(148, 163, 184, 0.3);
  }

  .app-layout {
    display: flex;
    height: 100vh;
  }

  .sidebar {
    width: 220px;
    background: linear-gradient(180deg, #0f1225 0%, #0c0f1a 100%);
    border-right: 1px solid rgba(148, 163, 184, 0.08);
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
    padding: 0.5rem;
    transition: width 0.2s ease;
    overflow: hidden;
  }

  .sidebar.collapsed {
    width: 52px;
  }

  .sidebar-brand {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.85rem 0.75rem;
    margin-bottom: 0.25rem;
  }

  .brand-icon {
    width: 22px;
    height: 22px;
    color: #818cf8;
    flex-shrink: 0;
  }

  .sidebar-brand span {
    font-size: 1.05rem;
    font-weight: 700;
    background: linear-gradient(135deg, #c7d2fe 0%, #818cf8 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    letter-spacing: -0.01em;
  }

  .nav-links {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
  }

  .nav-link {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.55rem 0.75rem;
    color: #94a3b8;
    text-decoration: none;
    border-radius: 8px;
    font-size: 0.875rem;
    font-weight: 500;
    transition: all 0.15s ease;
  }

  .nav-link svg {
    width: 18px;
    height: 18px;
    flex-shrink: 0;
  }

  .nav-link:hover {
    background: rgba(148, 163, 184, 0.08);
    color: #e2e8f0;
  }

  .nav-link.active {
    background: rgba(99, 102, 241, 0.12);
    color: #a5b4fc;
  }

  .nav-link.active svg {
    color: #818cf8;
  }

  .sidebar-footer {
    padding: 0.5rem;
    border-top: 1px solid rgba(148, 163, 184, 0.06);
    display: flex;
    justify-content: flex-end;
  }

  .collapse-btn {
    width: 28px;
    height: 28px;
    border-radius: 6px;
    border: none;
    background: transparent;
    color: #475569;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s ease;
  }

  .collapse-btn:hover {
    background: rgba(148, 163, 184, 0.08);
    color: #94a3b8;
  }

  .collapse-btn svg {
    width: 14px;
    height: 14px;
    transition: transform 0.2s ease;
  }

  .collapse-btn svg.flipped {
    transform: rotate(180deg);
  }

  .collapsed .sidebar-footer {
    justify-content: center;
  }

  .collapsed .sidebar-brand {
    justify-content: center;
  }

  .collapsed .nav-link {
    justify-content: center;
    padding: 0.55rem;
  }

  .main-content {
    flex: 1;
    overflow-y: auto;
    background: #0c0f1a;
  }

  .content-inner {
    max-width: 100%;
    height: 100%;
    padding: 1.5rem 2rem;
  }
</style>
