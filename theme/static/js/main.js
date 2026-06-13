document.getElementById('theme-toggle').addEventListener('click', () => {
    document.documentElement.classList.toggle('dark');
    localStorage.setItem('theme',
        document.documentElement.classList.contains('dark') ? 'dark' : 'light');
});

document.getElementById('hamburger').addEventListener('click', () => {
    document.querySelector('.nav-links').classList.toggle('nav-open');
    document.getElementById('hamburger').classList.toggle('open');
});

document.querySelectorAll('.tab-bar').forEach(bar => {
    bar.addEventListener('click', e => {
        const btn = e.target.closest('.tab-btn');
        if (!btn) return;
        const slug = btn.dataset.tab;
        const tabs = btn.closest('.tabs');
        tabs.querySelectorAll('.tab-btn').forEach(b => {
            b.classList.toggle('active', b.dataset.tab === slug);
            b.setAttribute('aria-selected', b.dataset.tab === slug);
        });
        tabs.querySelectorAll('.tab-panel').forEach(p => {
            p.classList.toggle('active', p.dataset.tab === slug);
        });
        history.replaceState(null, '', '#' + slug);
    });
});

if (location.hash) {
    const btn = document.querySelector('.tab-btn[data-tab="' + location.hash.slice(1) + '"]');
    if (btn) btn.click();
}
