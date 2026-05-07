import { h, type VNode } from "snabbdom";
import { coco, taxonomy, UserButton } from "@ratiosolver/coco";
import { App, flick, Navbar, NavbarItem, NavbarList, OffcanvasBrand } from "@ratiosolver/flick";

const landing_page = () => h('div.container.mt-5', [
  h('header.text-center.mb-5', [
    h('h1.display-4', 'ERMES'),
    h('p.lead.text-body-secondary', 'Ecosistema urbano per l\'invecchiamento attivo e in salute'),
  ]),
  h('div.row.justify-content-center', [
    h('div.col-lg-8', [
      h('p', 'ERMES (Urban Ecosystem for Active and Healthy Aging) mira a definire e implementare un modello di urbanizzazione innovativo e inclusivo, con una comunita residenziale per anziani dotata di servizi intelligenti basati su tecnologie ICT.'),
      h('hr.my-4'),
      h('h4', 'Obiettivi Principali'),
      h('ul.list-group.list-group-flush', [
        h('li.list-group-item', [h('i.fas.fa-city.me-2.text-primary'), h('strong', 'Urbanizzazione inclusiva'), ': realizzazione di spazi abitativi autonomi integrati nel tessuto urbano e sociale.']),
        h('li.list-group-item', [h('i.fas.fa-heart-pulse.me-2.text-primary'), h('strong', 'Integrazione socio-sanitaria'), ': collegamento tra servizi sanitari territoriali e remoti per supportare autonomia e benessere.']),
        h('li.list-group-item', [h('i.fas.fa-microchip.me-2.text-primary'), h('strong', 'Servizi intelligenti'), ': utilizzo di tecnologie innovative per sicurezza, assistenza e prevenzione delle patologie croniche legate all\'eta.']),
        h('li.list-group-item', [h('i.fas.fa-people-group.me-2.text-primary'), h('strong', 'Inclusione sociale'), ': promozione di spazi e attivita condivise per una vita attiva e partecipata.']),
      ]),
      h('hr.my-4'),
      h('p', [
        'Il modello ERMES e personalizzato sulle esigenze individuali e sulle caratteristiche del territorio, con focus sul comune di ', h('strong', 'Ginosa'), '. L\'ecosistema integra abitazioni, spazi condivisi e servizi di cura in una rete multidisciplinare orientata all\'invecchiamento attivo e in salute.',
      ]),
      h('p', [
        'Il progetto coinvolge partner scientifici, clinici e istituzionali per costruire un ecosistema sostenibile dedicato alla popolazione anziana, favorendo prevenzione, inclusione, sicurezza e autonomia nel lungo periodo.',
      ]),
      h('h5.mt-4', 'Partner del progetto'),
      h('ul.ps-3', [
        h('li', 'IRCCS INRCA (coordinatore)'),
        h('li', 'CNR - Consiglio Nazionale delle Ricerche'),
        h('li', 'Comune di Ginosa'),
        h('li', 'IRCCS Casa Sollievo della Sofferenza (CSS)'),
        h('li', 'IRCCS Fondazione Santa Lucia'),
        h('li', 'IRCCS Istituto delle Scienze Neurologiche di Bologna'),
        h('li', 'IRCCS NEUROMED - Istituto Neurologico Mediterraneo'),
        h('li', 'Università degli Studi di Firenze'),
        h('li', 'Sapienza Università di Roma'),
        h('li', 'Università degli Studi di Bologna'),
      ]),
    ])
  ])
]);

const connection_listener: coco.ConnectionListener = {
  connected: () => { },
  user_updated: (_user: coco.CoCoUser | null) => { },
  disconnected: () => {
    flick.ctx.current_page = landing_page;
    flick.ctx.page_title = 'Home';
    flick.redraw();
  },
  connection_error: (_error: Event) => { },
};

const coco_listener: coco.CoCoListener = {
  initialized: () => flick.redraw(),
  created_class: (_cls: coco.CoCoClass) => flick.redraw(),
  created_object: (_obj: coco.CoCoObject) => flick.redraw(),
  created_rule: (_rule: coco.CoCoRule) => flick.redraw(),
};

flick.ctx.current_page = landing_page;
flick.ctx.page_title = 'Home';

export function ERMESApp(coco: coco.CoCo): VNode {
  const content = h('div.flex-grow-1.d-flex.flex-column',
    {
      hook: {
        insert: () => {
          coco.add_connection_listener(connection_listener);
          coco.add_listener(coco_listener);
        },
        destroy: () => {
          coco.remove_connection_listener(connection_listener);
          coco.remove_listener(coco_listener);
        }
      }
    }, [
    (flick.ctx.current_page as () => VNode)()
  ]);

  return App(Navbar(OffcanvasBrand('ERMES'), [NavbarList([NavbarItem(h('i.fas.fa-home', {
    on: {
      click: () => {
        flick.ctx.current_page = landing_page;
        flick.ctx.page_title = 'Home';
        flick.redraw();
      }
    }
  })),
  NavbarItem(h('i.fas.fa-sitemap', {
    on: {
      click: () => {
        flick.ctx.current_page = () => taxonomy(coco);
        flick.ctx.page_title = 'Taxonomy';
        flick.redraw();
      }
    }
  }))]), UserButton(coco)]), content);
}