import { defineBehavior } from './index';

const THEME_KEY = 'kanidm-admin-ui:theme';

defineBehavior({
  selector: '[data-theme-set]',
  event: 'click',
  handler: (el, event) => {
    const theme = el.dataset.themeSet;
    if (theme !== 'light' && theme !== 'dark') return;
    event.preventDefault();
    document.documentElement.classList.toggle('dark', theme === 'dark');
    try {
      localStorage.setItem(THEME_KEY, theme);
    } catch {}
  },
});
