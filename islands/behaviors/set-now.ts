import { defineBehavior } from './index';

defineBehavior({
  selector: '[data-set-now]',
  event: 'click',
  handler: (el, event) => {
    event.preventDefault();
    const card = el.closest<HTMLElement>('[data-validity-card]');
    if (!card) return;
    const dateInput = card.querySelector<HTMLInputElement>('input[type=date]');
    const timeInput = card.querySelector<HTMLInputElement>('input[type=time]');
    const radioDatetime = card.querySelector<HTMLInputElement>('input[type=radio][value=datetime]');
    if (!dateInput || !timeInput) return;
    const now = new Date();
    dateInput.value = now.toISOString().slice(0, 10);
    timeInput.value = now.toISOString().slice(11, 16);
    if (radioDatetime) radioDatetime.checked = true;
  },
});
