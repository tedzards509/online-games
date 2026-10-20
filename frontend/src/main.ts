import { mount } from 'svelte';
import App from './App.svelte';
import '@fontsource-variable/nunito-sans';
import './lib/i18n';
import './design.css';
import './style.css';

mount(App, { target: document.getElementById('app')! });
