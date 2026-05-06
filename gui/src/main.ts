import { flick } from '@ratiosolver/flick';
import '@fortawesome/fontawesome-free/css/all.css';
import { coco, CoCoApp } from '@ratiosolver/coco';

const cc = new coco.CoCo();

flick.mount(() => CoCoApp(cc));
