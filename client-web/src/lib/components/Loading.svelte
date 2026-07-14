<script>
  // ZafafWorld - Loading Screen Component (Glassmorphism)
  // Usage: <Loading show={isLoading} />
  let { show = true } = $props();
</script>

{#if show}
<div class="zw-loader">
  <div class="glass-panel">
    <div class="logo-wrap">
      <div class="ring"></div>
      <div class="ring slow"></div>
      <div class="diamond-glow"></div>
      <img class="logo-img" src="/logo.webp" alt="ZafafWorld Logo" />
    </div>
    <div class="brand-text">ZAFAF WORLD</div>
    <div class="sub-text">
      <span>LOADING</span>
      <div class="dot"></div>
      <div class="dot"></div>
      <div class="dot"></div>
    </div>
  </div>
</div>
{/if}

<style>
  :root{
    --gold: #d4af37;
    --gold-light: #f7e7b0;
    --gold-dark: #9a7b1f;
  }

  /* Full-screen overlay: TRANSPARENT + blurred (glassmorphism) so the
     real page behind it shows through, softly frosted. */
  .zw-loader{
    position:fixed;
    top:0;left:0;right:0;bottom:0;
    display:flex;
    align-items:center;
    justify-content:center;
    z-index:99999;
    background: rgba(10,8,6,0.28);
    -webkit-backdrop-filter: blur(18px) saturate(140%);
    backdrop-filter: blur(18px) saturate(140%);
    animation: fadeIn 0.3s ease;
  }
  @keyframes fadeIn{
    from{ opacity:0; }
    to{ opacity:1; }
  }

  /* Frosted glass card holding the logo + text */
  .glass-panel{
    display:flex;
    flex-direction:column;
    align-items:center;
    padding: 40px 50px;
    border-radius: 24px;
    background: rgba(255,255,255,0.06);
    border: 1px solid rgba(255,255,255,0.18);
    box-shadow:
      0 8px 32px rgba(0,0,0,0.25),
      inset 0 1px 0 rgba(255,255,255,0.15);
    -webkit-backdrop-filter: blur(6px);
    backdrop-filter: blur(6px);
    font-family: 'Georgia', 'Times New Roman', serif;
  }

  .logo-wrap{
    position:relative;
    width:200px;
    height:200px;
    display:flex;
    align-items:center;
    justify-content:center;
  }
  .ring{
    position:absolute;
    top:0;left:0;
    width:200px;
    height:200px;
    border-radius:50%;
    border: 3px solid transparent;
    border-top-color: var(--gold);
    border-right-color: var(--gold-light);
    animation: spin 1.6s linear infinite;
    box-shadow: 0 0 18px rgba(212,175,55,0.55);
  }
  .ring.slow{
    width:170px;
    height:170px;
    top:15px;
    left:15px;
    border-top-color: transparent;
    border-bottom-color: var(--gold-dark);
    border-left-color: var(--gold);
    animation: spin-rev 2.4s linear infinite;
    box-shadow: none;
  }
  .logo-img{
    width:120px;
    height:120px;
    border-radius:50%;
    object-fit:contain;
    background: radial-gradient(circle, rgba(212,175,55,0.12) 0%, transparent 70%);
    animation: pulse 2s ease-in-out infinite;
    filter: drop-shadow(0 0 10px rgba(212,175,55,0.5));
    position:relative;
    z-index:2;
  }

  /* Glow that sits over the diamond at the top of the logo and spins */
  .diamond-glow{
    position:absolute;
    top:36px;
    left:50%;
    transform: translateX(-50%);
    width:36px;
    height:30px;
    border-radius:50%;
    mix-blend-mode: screen;
    pointer-events:none;
    z-index:3;
    filter: blur(0.5px);
  }
  .diamond-glow::before{
    content:'';
    position:absolute;
    inset:0;
    border-radius:50%;
    background: conic-gradient(
      from 0deg,
      rgba(255,255,255,0.95),
      rgba(255,215,120,0.15),
      rgba(255,255,255,0.9),
      rgba(255,215,120,0.15),
      rgba(255,255,255,0.95)
    );
    animation: diamond-spin 2.2s linear infinite;
  }
  .diamond-glow::after{
    content:'';
    position:absolute;
    inset:8px;
    border-radius:50%;
    background: radial-gradient(circle, rgba(255,255,255,0.95) 0%, rgba(255,255,255,0) 70%);
    animation: diamond-pulse 1.7s ease-in-out infinite;
  }

  @keyframes spin{ to{ transform: rotate(360deg); } }
  @keyframes spin-rev{ to{ transform: rotate(-360deg); } }
  @keyframes pulse{
    0%,100%{ transform: scale(1); }
    50%{ transform: scale(1.06); }
  }
  @keyframes diamond-spin{ to{ transform: rotate(360deg); } }
  @keyframes diamond-pulse{
    0%,100%{ opacity:0.5; transform:scale(0.8); }
    50%{ opacity:1; transform:scale(1.2); }
  }

  .brand-text{
    margin-top:22px;
    font-size:22px;
    letter-spacing:3px;
    color: var(--gold-light);
    text-transform:uppercase;
    text-shadow: 0 0 12px rgba(212,175,55,0.6);
  }
  .sub-text{
    margin-top:8px;
    font-size:13px;
    letter-spacing:2px;
    color: var(--gold-dark);
    display:flex;
    align-items:center;
    gap:4px;
  }
  .dot{
    width:5px;
    height:5px;
    border-radius:50%;
    background: var(--gold);
    animation: blink 1.4s infinite ease-in-out;
  }
  .dot:nth-child(2){ animation-delay:0.2s; }
  .dot:nth-child(3){ animation-delay:0.4s; }
  @keyframes blink{
    0%,80%,100%{ opacity:0.2; }
    40%{ opacity:1; }
  }
</style>
