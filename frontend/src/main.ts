import { mount } from 'svelte';
import App from './App.svelte';
import './lib/i18n';
import './style';

mount(App, { target: document.getElementById('app')! });
