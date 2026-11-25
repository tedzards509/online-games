/** Global styles: layout tokens, the active theme, and element defaults.
 * A theme scopes its light tokens to `:root[data-theme='<name>']` and its dark
 * color overrides to `:root[data-theme='<name>'][data-mode='dark']`; switching
 * themes means importing another theme file and changing the attribute.
 * `lib/colorMode` sets `data-mode` synchronously, before the first styled paint. */
import './design.css';
import './themes/inkline.css';
import './base.css';
import './lib/colorMode';

document.documentElement.dataset.theme = 'inkline';
