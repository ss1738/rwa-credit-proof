const scenarioButtons = [...document.querySelectorAll('[data-scenario]')];
for (const button of scenarioButtons) {
  button.addEventListener('click', () => {
    const swapped = button.dataset.scenario === 'swapped';
    const nativeDemo = document.querySelector('.binding-demo');
    nativeDemo.dataset.state = swapped ? 'swapped' : 'attested';
    nativeDemo.querySelector('.output-symbol').textContent = swapped ? '×' : '✓';
    for (const item of scenarioButtons) item.setAttribute('aria-pressed', String(item === button));
    const signature = document.querySelector('#signature-result');
    signature.textContent = swapped ? 'Fail' : 'Pass';
    signature.classList.toggle('fail', swapped);
    document.querySelector('#decision').classList.toggle('blocked', swapped);
    document.querySelector('#decision-result').textContent = swapped ? 'Model blocks' : 'Model allows ↗';
    document.querySelector('#scenario-explanation').textContent = swapped
      ? 'The different book can have a valid proof, but the original servicer signature does not match its commitment.'
      : 'The proof and signature bind the decision to the same attested book.';
  });
}
const copyButton = document.querySelector('#copy-demo');
copyButton.addEventListener('click', async () => {
  const status = document.querySelector('#copy-status');
  try {
    await navigator.clipboard.writeText('./demo.sh');
    copyButton.textContent = 'Copied';
    status.textContent = 'Demo command copied to clipboard.';
  } catch {
    copyButton.textContent = 'Select below';
    status.textContent = 'Clipboard unavailable. Select and copy ./demo.sh from the code block.';
  }
});

// Motion is illustrative. No wallet, network connection or proof generation.
const root = document.documentElement;
const motionPreference = window.matchMedia('(prefers-reduced-motion: reduce)');
const motionButton = document.querySelector('#motion-toggle');
const hero = document.querySelector('.hero');
let userPaused = false;
try { userPaused = sessionStorage.getItem('caledren-motion-paused') === 'true'; } catch { /* Storage is optional. */ }
let motionPaused = userPaused || motionPreference.matches;
let heroVisible = true;
let updateNetwork = () => {};
function syncMotion() {
  motionPaused = userPaused || motionPreference.matches;
  root.classList.toggle('motion-paused', motionPaused);
  motionButton.setAttribute('aria-pressed', String(motionPaused));
  motionButton.disabled = motionPreference.matches;
  motionButton.querySelector('.motion-label').textContent = motionPreference.matches ? 'Reduced motion' : motionPaused ? 'Play motion' : 'Pause motion';
  motionButton.setAttribute('aria-label', motionButton.querySelector('.motion-label').textContent);
  motionButton.firstElementChild.textContent = motionPaused ? '▷' : 'Ⅱ';
  motionButton.hidden = false;
  if (motionPaused) {
    document.querySelector('.proof-figure').style.removeProperty('--pointer-x');
    document.querySelector('.proof-figure').style.removeProperty('--pointer-y');
    for (const card of document.querySelectorAll('.metric')) {
      card.style.removeProperty('--card-rx');
      card.style.removeProperty('--card-ry');
    }
  }
  updateNetwork();
}
motionButton.addEventListener('click', () => {
  userPaused = !userPaused;
  try { sessionStorage.setItem('caledren-motion-paused', String(userPaused)); } catch { /* Storage is optional. */ }
  syncMotion();
});
motionPreference.addEventListener('change', syncMotion);
syncMotion();

const revealTargets = document.querySelectorAll('.section-heading, .metrics, .network-table, .pipeline-map, .steps article, .binding-demo, .audit-grid article, .reproduce, .scope-map, .limits-grid, .partner-section > div, .partner-aside');
if ('IntersectionObserver' in window) {
  const revealObserver = new IntersectionObserver(entries => {
    for (const entry of entries) {
      if (!entry.isIntersecting) continue;
      entry.target.classList.add('is-visible');
      revealObserver.unobserve(entry.target);
    }
  }, { threshold: 0.08, rootMargin: '0px 0px -20px 0px' });
  for (const target of revealTargets) {
    target.classList.add('reveal-ready');
    revealObserver.observe(target);
  }
  const heroObserver = new IntersectionObserver(([entry]) => {
    heroVisible = entry.isIntersecting;
    hero.classList.toggle('is-offscreen', !heroVisible);
    updateNetwork();
  }, { threshold: 0 });
  heroObserver.observe(hero);
  const sectionMotionObserver = new IntersectionObserver(entries => {
    for (const entry of entries) entry.target.classList.toggle('section-motion-offscreen', !entry.isIntersecting);
  }, { threshold: 0 });
  for (const section of document.querySelectorAll('main > section:not(.hero), .protocol-rail')) {
    section.classList.add('section-motion-offscreen');
    sectionMotionObserver.observe(section);
  }
}

// Recorded values stay fixed while their decorative traces respond to the pointer.
const finePointer = window.matchMedia('(hover: hover) and (pointer: fine)');
for (const card of document.querySelectorAll('.metric')) {
  card.addEventListener('pointermove', event => {
    if (motionPaused || !finePointer.matches || event.pointerType === 'touch') return;
    const rect = card.getBoundingClientRect();
    const x = Math.max(0, Math.min(1, (event.clientX - rect.left) / rect.width));
    const y = Math.max(0, Math.min(1, (event.clientY - rect.top) / rect.height));
    card.style.setProperty('--card-rx', `${((.5-y)*5).toFixed(2)}deg`);
    card.style.setProperty('--card-ry', `${((x-.5)*5).toFixed(2)}deg`);
    card.style.setProperty('--spot-x', `${(x*100).toFixed(1)}%`);
    card.style.setProperty('--spot-y', `${(y*100).toFixed(1)}%`);
  }, { passive:true });
  card.addEventListener('pointerleave', () => {
    card.style.removeProperty('--card-rx');
    card.style.removeProperty('--card-ry');
  });
}

const sectionLinks = [...document.querySelectorAll('.site-header nav a')];
const linkedSections = sectionLinks.map(link => document.querySelector(link.getAttribute('href')));
const readingProgress = document.querySelector('.reading-progress i');
let scrollFrame = 0;
function updateScrollState() {
  scrollFrame = 0;
  const maxScroll = document.documentElement.scrollHeight - window.innerHeight;
  const fraction = maxScroll > 0 ? Math.max(0, Math.min(1, window.scrollY / maxScroll)) : 0;
  let active = -1;
  // Navigation order can differ from document order; compare positions first.
  let closest = -Infinity;
  for (let index = 0; index < linkedSections.length; index++) {
    const top = linkedSections[index].getBoundingClientRect().top;
    if (top <= window.innerHeight * .35 && top > closest) { active = index; closest = top; }
  }
  readingProgress.style.transform = `scaleX(${fraction.toFixed(4)})`;
  const activeId = active >= 0 ? linkedSections[active].id : null;
  for (let index = 0; index < sectionLinks.length; index++) {
    if (linkedSections[index].id === activeId) sectionLinks[index].setAttribute('aria-current', 'location');
    else sectionLinks[index].removeAttribute('aria-current');
  }
}
function queueScrollState() {
  if (!scrollFrame) scrollFrame = requestAnimationFrame(updateScrollState);
}
window.addEventListener('scroll', queueScrollState, { passive:true });
window.addEventListener('resize', queueScrollState, { passive:true });
updateScrollState();

const mobileMenu = document.querySelector('.mobile-nav');
for (const link of mobileMenu.querySelectorAll('a')) {
  link.addEventListener('click', () => { mobileMenu.open = false; });
}
document.addEventListener('keydown', event => {
  if (event.key === 'Escape' && mobileMenu.open) {
    mobileMenu.open = false;
    mobileMenu.querySelector('summary').focus();
  }
});
document.addEventListener('click', event => {
  if (!mobileMenu.contains(event.target)) mobileMenu.open = false;
});

// Native anchor-jump lands short on this page (layout not yet settled at
// navigation time), so in-page links are scrolled explicitly instead.
// 'instant' is used deliberately: 'smooth' scrollIntoView gets interrupted
// here (the per-frame scroll listener below forces layout reads mid-animation)
// and lands at inconsistent positions, verified across all five section links.
function scrollToHash(hash) {
  if (!hash || hash.length < 2) return;
  const target = document.querySelector(hash);
  if (!target) return;
  target.scrollIntoView({ behavior: 'instant', block: 'start' });
}
document.addEventListener('click', event => {
  const link = event.target.closest('a[href^="#"]');
  if (!link || !document.querySelector(link.getAttribute('href'))) return;
  event.preventDefault();
  const hash = link.getAttribute('href');
  scrollToHash(hash);
  history.pushState(null, '', hash);
});
if (location.hash) {
  // Wait two frames so layout (fonts, canvas sizing) has settled before jumping.
  requestAnimationFrame(() => requestAnimationFrame(() => scrollToHash(location.hash)));
}

const canvas = document.querySelector('#proof-network');
const context = canvas.getContext('2d');
if (context) {
  const figure = document.querySelector('.proof-figure');
  const tau = Math.PI * 2;
  const rows = 12, columns = 22;
  const vertices = [], edges = [];
  // A deterministic spherical lattice, not randomized market/network data.
  for (let row = 0; row <= rows; row++) {
    const latitude = (row / rows) * Math.PI;
    for (let column = 0; column < columns; column++) {
      const longitude = (column / columns) * tau + (row % 2 ? Math.PI / columns : 0);
      vertices.push({ x: Math.sin(latitude) * Math.cos(longitude), y: Math.cos(latitude), z: Math.sin(latitude) * Math.sin(longitude) });
      const index = row * columns + column;
      if (row > 0 && row < rows) edges.push([index, row * columns + (column + 1) % columns]);
      if (row > 0) edges.push([index, index - columns]);
      if (row > 1 && row < rows && column % 2 === 0) edges.push([index, (row - 1) * columns + (column + 1) % columns]);
    }
  }
  let width = 1, height = 1, pixelRatio = 1;
  let frameId = 0, lastFrame = 0, animationTime = 0;
  let pointerX = 0, pointerY = 0, easedX = 0, easedY = 0;
  const running = () => !motionPaused && heroVisible && !document.hidden;
  function project(vertex, angle, tilt) {
    const x = vertex.x * Math.cos(angle) + vertex.z * Math.sin(angle);
    const z = -vertex.x * Math.sin(angle) + vertex.z * Math.cos(angle);
    const y = vertex.y * Math.cos(tilt) - z * Math.sin(tilt);
    const depth = vertex.y * Math.sin(tilt) + z * Math.cos(tilt);
    const perspective = 3.8 / (3.8 - depth);
    const radius = Math.min(width * .35, height * .33);
    return { x: width * .5 + x * radius * perspective, y: height * (width < 360 ? .46 : .48) + y * radius * perspective, z: depth };
  }
  function draw(time) {
    context.setTransform(pixelRatio, 0, 0, pixelRatio, 0, 0);
    context.clearRect(0, 0, width, height);
    easedX += (pointerX - easedX) * .04;
    easedY += (pointerY - easedY) * .04;
    const angle = time * .115 + easedX * .25;
    const tilt = .26 + Math.sin(time * .11) * .12 + easedY * .18;
    const points = vertices.map(vertex => project(vertex, angle, tilt));
    const center = project({ x:0, y:0, z:0 }, angle, tilt);
    const glow = context.createRadialGradient(center.x, center.y, 5, center.x, center.y, width * .49);
    glow.addColorStop(0, 'rgba(149, 239, 70, 0.055)');
    glow.addColorStop(.58, 'rgba(122, 211, 61, 0.025)');
    glow.addColorStop(1, 'rgba(122, 211, 61, 0)');
    context.fillStyle = glow;
    context.fillRect(0, 0, width, height);
    // Data packets converge on the proof core, then leave toward the public claim.
    const paths = [
      [{ x:width * .24, y:height * .27 }, { x:width * .10, y:height * .53 }, center],
      [center, { x:width * .86, y:height * .38 }, { x:width * .82, y:height * .73 }]
    ];
    for (let pathIndex = 0; pathIndex < paths.length; pathIndex++) {
      const [start, bend, end] = paths[pathIndex];
      context.beginPath();
      context.moveTo(start.x, start.y);
      context.quadraticCurveTo(bend.x, bend.y, end.x, end.y);
      context.strokeStyle = 'rgba(200, 255, 122, 0.13)';
      context.lineWidth = .7;
      context.stroke();
      for (let packet = 0; packet < 3; packet++) {
        for (let tail = 7; tail >= 0; tail--) {
          const phase = (time * .15 + packet / 3 + pathIndex * .5 - tail * .007 + 2) % 1;
          const x = (1-phase)**2 * start.x + 2*(1-phase)*phase * bend.x + phase**2 * end.x;
          const y = (1-phase)**2 * start.y + 2*(1-phase)*phase * bend.y + phase**2 * end.y;
          context.fillStyle = `rgba(221, 255, 162, ${(1-tail/8) * .85})`;
          context.beginPath();
          context.arc(x, y, tail === 0 ? 1.8 : 1, 0, tau);
          context.fill();
        }
      }
    }
    for (const [a, b] of edges) {
      const depth = (points[a].z + points[b].z) / 2;
      const alpha = .035 + ((depth + 1) / 2) * .24;
      context.beginPath();
      context.moveTo(points[a].x, points[a].y);
      context.lineTo(points[b].x, points[b].y);
      context.strokeStyle = `rgba(169, 227, 117, ${alpha})`;
      context.lineWidth = depth > .4 ? .75 : .5;
      context.stroke();
    }
    for (let index = columns; index < points.length - columns; index++) {
      const point = points[index];
      const brightness = (point.z + 1) / 2;
      const pulse = .65 + .35 * Math.sin(time * 1.4 + index * .7);
      context.beginPath();
      context.arc(point.x, point.y, .65 + brightness * .9, 0, tau);
      context.fillStyle = `rgba(205, 255, 142, ${(.15 + brightness * .65) * pulse})`;
      context.fill();
    }
    // Comets move across the lattice; the bright point leaves a short curved trail.
    for (let orbit = 0; orbit < 3; orbit++) {
      const latitude = .8 + orbit * .6;
      for (let trail = 15; trail >= 0; trail--) {
        const longitude = time * (.3 + orbit * .08) + orbit * 2.2 - trail * .018;
        const point = project({ x:Math.sin(latitude) * Math.cos(longitude) * 1.015, y:Math.cos(latitude) * 1.015, z:Math.sin(latitude) * Math.sin(longitude) * 1.015 }, angle, tilt);
        context.beginPath();
        context.arc(point.x, point.y, trail === 0 ? 2.7 : 1.4, 0, tau);
        context.fillStyle = orbit === 1 ? `rgba(119, 244, 200, ${(1 - trail / 16) * .9})` : `rgba(221, 255, 162, ${(1 - trail / 16) * .95})`;
        context.fill();
      }
    }
    // A few quiet points give the field depth without filling the page with noise.
    for (let i = 0; i < 28; i++) {
      const x = ((i * .618033) % 1) * width;
      const y = ((i * .381966) % 1) * height;
      const alpha = .12 + .13 * (.5 + .5 * Math.sin(time * .5 + i));
      context.fillStyle = `rgba(193, 224, 164, ${alpha})`;
      context.fillRect(x, y, i % 4 === 0 ? 2 : 1, 1);
    }
  }
  function tick(now) {
    frameId = 0;
    if (!running()) return;
    const elapsed = now - lastFrame;
    if (elapsed >= 32) {
      animationTime += Math.min(elapsed, 65) / 1000;
      lastFrame = now;
      draw(animationTime);
    }
    frameId = requestAnimationFrame(tick);
  }
  updateNetwork = () => {
    if (frameId) cancelAnimationFrame(frameId);
    frameId = 0;
    if (motionPaused) { pointerX = 0; pointerY = 0; easedX = 0; easedY = 0; }
    if (running()) {
      lastFrame = performance.now();
      frameId = requestAnimationFrame(tick);
    } else if (heroVisible && !document.hidden) draw(animationTime);
  };
  function resize() {
    const bounds = figure.getBoundingClientRect();
    width = bounds.width;
    height = bounds.height;
    pixelRatio = Math.min(window.devicePixelRatio || 1, 1.5);
    canvas.width = Math.round(width * pixelRatio);
    canvas.height = Math.round(height * pixelRatio);
    draw(animationTime);
    updateNetwork();
  }
  if ('ResizeObserver' in window) new ResizeObserver(resize).observe(figure);
  else window.addEventListener('resize', resize, { passive:true });
  hero.addEventListener('pointermove', event => {
    if (motionPaused || event.pointerType === 'touch') return;
    const bounds = hero.getBoundingClientRect();
    pointerX = Math.max(-1, Math.min(1, (event.clientX - bounds.left) / bounds.width * 2 - 1));
    pointerY = Math.max(-1, Math.min(1, (event.clientY - bounds.top) / bounds.height * 2 - 1));
    figure.style.setProperty('--pointer-x', pointerX.toFixed(3));
    figure.style.setProperty('--pointer-y', pointerY.toFixed(3));
  }, { passive:true });
  hero.addEventListener('pointerleave', () => {
    pointerX = 0; pointerY = 0;
    figure.style.setProperty('--pointer-x', '0');
    figure.style.setProperty('--pointer-y', '0');
  });
  document.addEventListener('visibilitychange', () => {
    root.classList.toggle('page-hidden', document.hidden);
    updateNetwork();
  });
  resize();
}
