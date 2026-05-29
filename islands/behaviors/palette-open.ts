import { defineBehavior } from './index';

defineBehavior({
  selector: '[data-open-palette]',
  event: 'click',
  handler: (_el, event) => {
    event.preventDefault();
    document.dispatchEvent(new CustomEvent('open-palette'));
  },
});
