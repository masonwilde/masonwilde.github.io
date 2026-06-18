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
    const slug = CSS.escape(location.hash.slice(1));
    const btn = document.querySelector('.tab-btn[data-tab="' + slug + '"]');
    if (btn) btn.click();
}

(function () {
    var overlay = document.getElementById('search-overlay');
    var input = document.getElementById('search-input');
    var resultsEl = document.getElementById('search-results');
    var toggle = document.getElementById('search-toggle');
    if (!overlay || !input || !toggle) return;

    var searchData = null;
    var fuse = null;
    var selectedIdx = -1;

    function loadIndex() {
        if (searchData) return Promise.resolve();
        return fetch('/search-index.json')
            .then(function (r) { return r.json(); })
            .then(function (data) {
                searchData = data;
                fuse = new Fuse(data, {
                    keys: [
                        { name: 'title', weight: 3 },
                        { name: 'description', weight: 2 },
                        { name: 'content', weight: 1 }
                    ],
                    threshold: 0.3,
                    includeScore: true,
                    minMatchCharLength: 2
                });
            });
    }

    function parseQuery(query) {
        var categories = [];
        var remaining = query.replace(/category:(\S+)/gi, function (_, cat) {
            categories.push(cat.toLowerCase());
            return '';
        }).trim();

        var orGroups = remaining.split(/\s+OR\s+/).map(function (group) {
            return group.split(/\s+/).filter(function (t) {
                return t && t.toUpperCase() !== 'AND';
            });
        }).filter(function (g) { return g.length > 0; });

        return { categories: categories, orGroups: orGroups };
    }

    function doSearch(query) {
        if (!fuse || !query.trim()) {
            resultsEl.textContent = '';
            selectedIdx = -1;
            return;
        }

        var parsed = parseQuery(query);
        var data = searchData;

        if (parsed.categories.length > 0) {
            data = data.filter(function (d) {
                return parsed.categories.indexOf((d.section || '').toLowerCase()) !== -1;
            });
        }

        if (parsed.orGroups.length === 0) {
            resultsEl.textContent = '';
            selectedIdx = -1;
            return;
        }

        var searchFuse = parsed.categories.length > 0
            ? new Fuse(data, fuse.options)
            : fuse;

        var results = [];

        for (var g = 0; g < parsed.orGroups.length; g++) {
            var andGroup = parsed.orGroups[g];
            var groupResults = null;

            for (var t = 0; t < andGroup.length; t++) {
                var termResults = searchFuse.search(andGroup[t]);

                if (groupResults === null) {
                    groupResults = termResults;
                } else {
                    var termScores = {};
                    for (var i = 0; i < termResults.length; i++) {
                        termScores[termResults[i].item.url] = termResults[i].score;
                    }
                    groupResults = groupResults.filter(function (r) {
                        return termScores.hasOwnProperty(r.item.url);
                    }).map(function (r) {
                        var other = termScores[r.item.url];
                        return { item: r.item, score: Math.max(r.score, other) };
                    });
                }
            }
            if (groupResults) {
                for (var i = 0; i < groupResults.length; i++) {
                    results.push(groupResults[i]);
                }
            }
        }

        var seen = {};
        var deduped = [];
        for (var i = 0; i < results.length; i++) {
            var r = results[i];
            if (!seen[r.item.url] || seen[r.item.url].score > r.score) {
                seen[r.item.url] = r;
            }
        }
        for (var url in seen) {
            deduped.push(seen[url]);
        }
        deduped.sort(function (a, b) { return a.score - b.score; });

        selectedIdx = -1;
        renderResults(deduped.slice(0, 10));
    }

    function renderResults(results) {
        resultsEl.textContent = '';
        input.setAttribute('aria-expanded', results.length > 0 ? 'true' : 'false');
        if (results.length === 0) {
            var empty = document.createElement('div');
            empty.className = 'search-no-results';
            empty.textContent = 'No results found';
            resultsEl.appendChild(empty);
            return;
        }

        for (var i = 0; i < results.length; i++) {
            var item = results[i].item;
            var link = document.createElement('a');
            link.href = item.url;
            link.className = 'search-result';
            link.setAttribute('role', 'option');
            link.id = 'search-result-' + i;
            link.dataset.idx = i;

            var titleRow = document.createElement('div');
            titleRow.className = 'search-result-title';
            titleRow.textContent = item.title;

            if (item.section) {
                var badge = document.createElement('span');
                badge.className = 'search-result-section';
                badge.textContent = item.section;
                titleRow.appendChild(badge);
            }
            link.appendChild(titleRow);

            if (item.description) {
                var desc = document.createElement('div');
                desc.className = 'search-result-desc';
                desc.textContent = item.description;
                link.appendChild(desc);
            }
            resultsEl.appendChild(link);
        }
    }

    function updateSelection() {
        var items = resultsEl.querySelectorAll('.search-result');
        for (var i = 0; i < items.length; i++) {
            items[i].classList.toggle('selected', i === selectedIdx);
            items[i].setAttribute('aria-selected', i === selectedIdx ? 'true' : 'false');
        }
        input.setAttribute('aria-activedescendant',
            selectedIdx >= 0 ? 'search-result-' + selectedIdx : '');
        if (selectedIdx >= 0 && items[selectedIdx]) {
            items[selectedIdx].scrollIntoView({ block: 'nearest' });
        }
    }

    function openSearch() {
        overlay.classList.add('active');
        input.focus();
        loadIndex();
    }

    function closeSearch() {
        overlay.classList.remove('active');
        input.value = '';
        input.setAttribute('aria-expanded', 'false');
        resultsEl.textContent = '';
        selectedIdx = -1;
    }

    toggle.addEventListener('click', openSearch);

    overlay.addEventListener('click', function (e) {
        if (e.target === overlay) closeSearch();
    });

    input.addEventListener('input', function () {
        doSearch(input.value);
    });

    input.addEventListener('keydown', function (e) {
        var items = resultsEl.querySelectorAll('.search-result');
        if (e.key === 'Escape') {
            closeSearch();
        } else if (e.key === 'ArrowDown') {
            e.preventDefault();
            if (selectedIdx < items.length - 1) selectedIdx++;
            updateSelection();
        } else if (e.key === 'ArrowUp') {
            e.preventDefault();
            if (selectedIdx > 0) selectedIdx--;
            updateSelection();
        } else if (e.key === 'Enter' && selectedIdx >= 0 && items[selectedIdx]) {
            closeSearch();
            window.location.href = items[selectedIdx].getAttribute('href');
        }
    });

    document.addEventListener('keydown', function (e) {
        if (e.key === '/' && !overlay.classList.contains('active')
            && document.activeElement.tagName !== 'INPUT'
            && document.activeElement.tagName !== 'TEXTAREA') {
            e.preventDefault();
            openSearch();
        }
    });
})();
