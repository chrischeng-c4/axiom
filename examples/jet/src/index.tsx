import { App } from './App';
import { createStore } from './utils/store';
import './styles/reset.css';
import './styles/app.css';

// Bootstrap
const root = document.getElementById('root');
if (!root) throw new Error('#root not found');

const store = createStore();
const app = App({ store });

root.innerHTML = app;

console.log('[warp-example] Todo app mounted');
