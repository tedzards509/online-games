/** Global styles: layout tokens, the active theme, and element defaults.
 * A theme scopes its tokens to `:root[data-theme='<name>']`; switching themes
 * means importing another theme file and changing the attribute. */
import './design.css';
import './themes/inkline.css';
import './base.css';

document.documentElement.dataset.theme = 'inkline';
