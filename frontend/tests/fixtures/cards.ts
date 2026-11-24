import { mount } from 'svelte';
import '../../src/lib/i18n';
import '../../src/style';
import Cards from './Cards.svelte';

mount(Cards, { target: document.getElementById('cards')! });
