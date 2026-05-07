import { flick } from '@ratiosolver/flick';
import '@fortawesome/fontawesome-free/css/all.css';
import { coco } from '@ratiosolver/coco';
import { ERMESApp } from './app';

const cc = new coco.CoCo();

flick.mount(() => ERMESApp(cc));
