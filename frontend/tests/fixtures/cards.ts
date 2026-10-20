import { mount } from 'svelte';
import '@fontsource-variable/nunito-sans';
import '../../src/lib/i18n';
import '../../src/design.css';
import '../../src/style.css';
import Cards from './Cards.svelte';

mount(Cards, { target: document.getElementById('cards')! });
