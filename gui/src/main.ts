import { flick } from '@ratiosolver/flick';
import '@fortawesome/fontawesome-free/css/all.css';
import { coco, CoCoApp } from '@ratiosolver/coco';

const cc = new coco.CoCo({ url: 'ws://localhost:3000/ws' });

flick.mount(() => CoCoApp(cc));

cc.connect();