# ERMES
**Ecosistema urbano per l'invecchiamento attivo e in salute**  
*Urban Ecosystem for Active and Healthy Aging*

### Project Description

The **ERMES** project aims to define and implement an innovative and inclusive urbanization model that includes the creation of a residential community for the elderly equipped with intelligent services through ICT technologies. The project is integrated with the urban fabric, social context, and health services, both local and remote, and is aimed at promoting and maintaining maximum autonomy from the perspective of active and healthy aging.

ERMES proposes an urbanization model customized based on individual needs and the specific characteristics of the urban territory, providing for the creation of autonomous living spaces in the urban community of **Ginosa (CS)**, equipped with "intelligent" products and services and shared care and recreational spaces, integrated in a socio-health model aimed at ensuring inclusion, security, wellbeing, health and assistance for the elderly population.

### Keywords

- Active and Healthy Ageing
- Urbanization
- Innovative technologies
- Social inclusion

### Expected Results

- ✅ Implementation of new care strategies based on a multidisciplinary network of socio-health services
- ✅ Creation of an inclusive and sustainable ecosystem aimed at promoting elderly autonomy, prevention and management of age-related chronic diseases

### Project Information

| | |
|---|---|
| **Coordinator** | IRCCS INRCA |
| **Scientific Director** | Prof. Fabrizia Lattanzio |
| **Research Area** | Socio-Economic Gerontology |
| **Total Budget** | € 4,815,000 |
| **INRCA Budget** | € 1,200,000 |
| **Duration** | 02/13/2023 - 02/12/2027 |

### Project Partners

- **CNR** - National Research Council
- **Municipality of Ginosa**
- **Casa Sollievo della Sofferenza [CSS]** – IRCCS
- **IRCCS - Fondazione Santa Lucia**
- **IRCCS Institute of Neurological Sciences of Bologna**
- **IRCCS NEUROMED** – Mediterranean Neurological Institute
- **University of Florence**
- **Sapienza University of Rome**
- **University of Bologna**

---

## System Startup (Docker)

This setup expects a local image named `ermes:latest`.

### 1) Build the ERMES image

Run from the repository root:

```bash
docker build -t ermes:latest .
```

### 2) Start the stack with Docker Compose

After building the image, choose one deployment profile.

Option A: public ports exposed (no Nginx Proxy Manager)

```bash
docker compose --profile public up -d
```

Option B: Nginx Proxy Manager integration

```bash
docker compose --profile npm up -d
```

The `npm` profile requires an existing external Docker network named `npm-network` (or the network used by your NPM instance).

### 3) Useful commands

View logs:

```bash
docker compose logs -f
```

Stop services:

```bash
docker compose down
```

### Profile behavior summary

1. `public` profile
- Starts `ermes-public` and exposes port `3000:3000`.
- Use this when ERMES is accessed directly without NPM.

2. `npm` profile
- Starts `ermes-npm` and connects it to both `ermes-network` and `npm-network`.
- Does not publish public ports for ERMES.
- Use this when traffic is routed by Nginx Proxy Manager.

---

## License

See [LICENSE](LICENSE) file for details.

---

*Project funded as part of the Active and Healthy Aging research program.*
