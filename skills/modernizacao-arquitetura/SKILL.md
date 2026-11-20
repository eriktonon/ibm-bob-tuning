---
name: modernizacao-arquitetura
description: "Guia completo para modernizacao de arquitetura e. Use quando: modernizar aplicacao existente."
user-invocable: true
---

# Skill de Modernização de Arquitetura

Esta skill centraliza o conhecimento necessário para conduzir **modernizações de arquitetura** e **migrações de linguagens legadas** no ecossistema PE-AGENTS-IBM. Ela deve ser consumida pelo agente `pe-agents-ibm-arquitetura` quando detectar contexto de modernização, e pelo agente `pe-agents-ibm-modernizacao` como principal fonte de decisão.

---

## Quando Usar

- O usuário pede para **modernizar uma aplicação existente**.
- O usuário quer **migrar uma linguagem defasada** (Java 8, .NET Framework, PHP antigo, Python 2, etc.).
- O projeto precisa sair de um **monolito para monólito modular ou microsserviços**.
- Há necessidade de documentar uma **decisão de modernização** em ADR.
- É necessário **estimar esforço e riscos** de uma migração.

---

## O que NÃO é escopo desta skill

- Não substitui o agente de arquitetura para decisões de nova aplicação.
- Não executa migração automaticamente — ela **prescreve o plano**.
- Não decide sozinha sobre schema de banco — delega para `pe-agents-ibm-banco-dados`.
- Não implementa código — delega para o agente backend especializado.

---

## Matriz de Decisão: Refatorar vs. Reescrever

| Critério | Refatorar / Strangler Fig | Reescrever / Rebuild |
|---|---|---|
| Cobertura de testes | ≥ 60% | < 30% |
| Domínio conhecido e documentado | Sim | Não / obscuro |
| Equipe conhece destino | Sim | Não (curva de aprendizado) |
| Prazo | < 6 meses | > 6 meses aceitável |
| Acoplamento a runtime proprietário | Baixo | Alto (Windows-only, mainframe, VB6) |
| Necessidade de manter operação 24x7 | Sim (zero downtime) | Pode aceitar cutover |
| Dívida técnica generalizada | Moderada | Extrema |

> **Regra geral**: prefira **Strangler Fig** para sistemas grandes e críticos; prefira **Rebuild** apenas quando o domínio for simples e a dívida técnica for irreversível.

---

## Catálogo de Migração por Linguagem / Stack

### Java

| Origem | Destino recomendado | Motivação | Riscos | Ferramentas | Estratégia |
|---|---|---|---|---|---|
| Java 8 | Java 21 LTS + Spring Boot 3.x | LTS, virtual threads, melhorias de GC | Jakarta EE namespace, remoção de APIs | OpenRewrite, Moderne, jdeps | Incremental |
| Java 8/11 | Java 21 + Quarkus | Cloud-native, startup rápido, memória baixa | Mudança de mindset, extension ecosystem | Quarkus migration guide | Strangler Fig |
| Spring Boot 2.x | Spring Boot 3.x | Suporte, Jakarta, Spring 6 | Breaking changes, security config | OpenRewrite recipes | Incremental |
| JEE / EJB | Quarkus / Spring Boot + microservices | Modernização, containerização | Modelo de transação, JNDI | Anti-Corruption Layer | Strangler Fig |

### .NET

| Origem | Destino recomendado | Motivação | Riscos | Ferramentas | Estratégia |
|---|---|---|---|---|---|
| .NET Framework 4.x | .NET 8/9/10 | Cross-platform, performance, LTS | WCF, Web Forms, Windows-only libs | .NET Upgrade Assistant, Upgrade Analyzer | Strangler Fig + ACL |
| ASP.NET Web Forms | Blazor / ASP.NET Core MVC / Minimal APIs | Modern web stack | Event-driven UI model, ViewState | Rebuild por página | Strangler Fig |
| WCF | gRPC / CoreWCF / REST APIs | Comunicação moderna | Bindings complexos, config XML | CoreWCF wrapper | ACL + wrapper |
| VB6 / VB.NET | C# + .NET 10 | Suporte, nuvem, recrutamento | COM, forms, controles | Rebuild com ACL | Rebuild |

### Node.js / TypeScript

| Origem | Destino recomendado | Motivação | Riscos | Ferramentas | Estratégia |
|---|---|---|---|---|---|
| Express legado | Fastify / NestJS | Performance 2-3x, tipagem, modularidade | Callbacks, middlewares acoplados | jscodeshift, ts-migrate | Replatform |
| JavaScript ES5 | TypeScript 5.x | Type safety, DX moderna | Build config, any-types | ts-migrate, jscodeshift | Refactor |
| Callbacks / then() | async/await | Legibilidade, tratamento de erro | Promise hell, race conditions | jscodeshift, eslint | Refactor |
| CommonJS | ESM | Tree-shaking, compatibilidade moderna | require dinâmico | jscodeshift, esbuild | Refactor |

### Python

| Origem | Destino recomendado | Motivação | Riscos | Ferramentas | Estratégia |
|---|---|---|---|---|---|
| Python 2.x | Python 3.11+ / FastAPI | EOL, async, type hints | print, divisão inteira, encoding | 2to3, pyupgrade, ruff | Replatform |
| Django antigo | Django 5.x / FastAPI | LTS, async views, modern ORM | Middleware, signals, templates | django-upgrade, ruff | Incremental |
| Flask sem tipagem | FastAPI + Pydantic v2 | OpenAPI automático, validação | Blueprints, extensões legadas | Manual + tests | Replatform |
| requirements.txt | pyproject.toml + uv/poetry | Reprodutibilidade, lock | Build backend, pins | poetry, uv | Refactor |

### PHP

| Origem | Destino recomendado | Motivação | Riscos | Ferramentas | Estratégia |
|---|---|---|---|---|---|
| PHP 5.x/7.x | PHP 8.3 + Laravel 11 | JIT, enums, typed properties | Extensões nativas, magic methods | Rector | Incremental |
| Laravel antigo | Laravel 11 | Novos defaults, performance, segurança | Quebras de pacotes | Laravel Shift / Rector | Incremental |
| CodeIgniter 3 | Laravel 11 / Symfony 7 | Arquitetura moderna | MVC acoplado | Rebuild por módulo | Strangler Fig |

### Outras linguagens legadas

| Origem | Destino recomendado | Motivação | Riscos | Ferramentas | Estratégia |
|---|---|---|---|---|---|
| Ruby on Rails antigo | Rails 8 / Hanami / API-only | Zeitwerk, performance, segurança | Monolito acoplado, gems mortas | rails_app_updater | Modular monolith |
| COBOL / mainframe | Java/C# / APIs wrapper | Escassez de mão de obra, custo | Regras implícitas, VSAM/DB2, batch | Micro Focus / Astadia | ACL + wrapper |
| Perl 5 | Python 3.11+ / Go | Legibilidade, recrutamento | Regex densas, CPAN | Rebuild por domínio | Rebuild |
| C++98/03 | C++20/23 / Rust | Segurança de memória, performance | UB, gerenciamento manual | clang-tidy, rust-bindgen | Refactor / FFI |
| Fortran 77/90 | Python (NumPy/SciPy) / C++ | HPC moderno, interoperabilidade | Arrays estáticos, COMMON | f2py, ISO_C_BINDING | Replatform |
| Delphi / Pascal | .NET / Java / Go | Suporte, integração | VCL/FMX, eventos | Rebuild com wrapper | Strangler Fig |
| SAP ABAP | SAP BTP / CAP / REST APIs | S/4HANA, cloud extension | Custom code readiness | ATC, RAP/CAP | Side-by-side |

---

## Padrões Arquiteturais de Modernização

| Padrão | Quando usar | Riscos | Ferramentas |
|---|---|---|---|
| **Strangler Fig** | Sistema grande/crítico sem testes completos; migração gradual | Duplicação temporária de dados; consistência eventual | Feature flags, proxy/ingress, eventos |
| **Anti-Corruption Layer (ACL)** | Legado com modelo de domínio tóxico ou incompatível | Overhead de tradução; latência | Adapters, translators, bounded contexts |
| **API Wrapper / Facade** | Expor funcionalidade legada sem alterá-la | Acoplamento ao legado; contrato frágil | OpenAPI, gateway, BFF |
| **Modular Monolith** | Monolito maduro; equipe quer independência sem microsserviços | Modularização incorreta (módulos acoplados) | DDD, packages/modules |
| **Event-Driven Replatforming** | Fluxos assíncronos; desacoplamento necessário | Complexidade operacional; exactly-once | Kafka, RabbitMQ, EventBridge |
| **Lift-and-Shift (containerizado)** | Pressa; pouca mudança de código; rehost na nuvem | Não resolve dívida técnica; custo de licença | Docker, cloud migration tools |
| **5R Continuum** | Framework de decisão: Rehost → Replatform → Refactor → Rearchitect → Rebuild | Escolher "R" errado gera custo ou falha | Matriz de decisão por workload |

---

## Ferramentas de Migração Automatizada

| Ferramenta | Domínio | Tipo | Observação |
|---|---|---|---|
| OpenRewrite / Moderne | Java, Kotlin, Maven/Gradle | Refatoração massiva por recipes | Suporta Java 8→21, Spring Boot 2→3, Quarkus |
| jscodeshift | JavaScript/TypeScript | Codemods | Padrões comuns Express→Fastify/NestJS |
| ts-migrate | JavaScript → TypeScript | Conversão de tipagem | Útil para grandes codebases JS |
| pyupgrade / 2to3 / Ruff | Python | Upgrade sintático | Python 2→3 e modernização |
| Rector | PHP | Refatoração AST | Migrações de versão PHP e Laravel |
| clang-tidy / Clangd | C/C++ | Linter/refator | Moderniza C++; não reescreve automaticamente |
| CodeQL | Múltiplas | Análise estática / segurança | Detecta padrões legados e vulnerabilidades |
| SonarQube / SonarCloud | Múltiplas | Qualidade contínua | Dívida técnica, code smells, coverage |
| JHipster | Java | Gerador de aplicação | Moderniza stack Java + frontend |
| .NET Upgrade Assistant | .NET | Migração assistida | Microsoft oficial para .NET Framework → .NET Core/5+ |
| Roslyn analyzers | C# | Análise e refactoring | Modernização de C# legado |
| AWS/Azure/GCP migration tools | Infra / workloads | Rehost/replatform | MGN, Azure Migrate, Migrate for Compute |
| LangChain / AST parsers custom | Legacy code analysis | Agentes sobre código | Análise semântica + plano; requer validação humana |

---

## Checklist de Assessment de Modernização

Antes de propor qualquer migração, responda:

- [ ] Qual é a linguagem/stack atual e a versão exata?
- [ ] Qual o destino desejado ou recomendado?
- [ ] Qual o tamanho aproximado do codebase (KLOC)?
- [ ] Qual a cobertura de testes atual?
- [ ] Existem testes de integração, E2E ou contrato?
- [ ] O sistema é crítico e deve manter disponibilidade 24x7?
- [ ] Há dependências de runtime proprietário (Windows, mainframe, COM, etc.)?
- [ ] O domínio de negócio está bem documentado ou mapeado?
- [ ] A equipe tem expertise no destino?
- [ ] Qual o prazo e orçamento disponíveis?
- [ ] Há requisitos de compliance (LGPD, PCI, SOC2) que impactam a migração?
- [ ] Existem dados sensíveis que exigem criptografia/mascaramento na migração?

---

## Template de Plano de Migração

```markdown
# Plano de Modernização — [Sistema]

## 1. Contexto
- Stack atual: [linguagem/versão/framework]
- Stack destino: [linguagem/versão/framework]
- Tamanho estimado: [KLOC]
- Cobertura de testes: [%]

## 2. Estratégia escolhida
- Padrão: [Strangler Fig / ACL / Rebuild / Modular Monolith / etc.]
- Justificativa: [por que este padrão]

## 3. Fases
| Fase | Escopo | Entregável | Ferramentas | Esforço estimado |
|---|---|---|---|---|
| 1 | Assessment | Inventário de dependências, testes, riscos | SonarQube, CodeQL | X dias |
| 2 | Preparação | Pipeline CI/CD, testes de baseline, feature flags | — | X dias |
| 3 | Migração X | Módulo/camada específica | [ferramenta] | X dias |
| 4 | Validação | Tester, segurança, integração | Tester, Segurança | X dias |
| 5 | Cutover / rollout | Release gradual, monitoramento | SRE | X dias |

## 4. Riscos e mitigações
| Risco | Probabilidade | Impacto | Mitigação |
|---|---|---|---|
| [risco] | Alta/Média/Baixa | Alto/Médio/Baixo | [ação] |

## 5. Critérios de sucesso
- [ ] Build e testes passando
- [ ] Cobertura de testes mantida ou aumentada
- [ ] Segurança sem CVEs críticas/alta
- [ ] Performance igual ou melhor que o legado
- [ ] Rollback testado e documentado
```

---

## Integração com Agente de Arquitetura

Quando `pe-agents-ibm-arquitetura` detectar pedido de modernização:

1. Executar a **Entrevista Obrigatória** do agente de arquitetura.
2. Aplicar esta skill para selecionar par legado→moderno, padrão e ferramentas.
3. Usar a skill `decisao-arquitetura` para definir estilo estrutural (monolito modular, microsserviços, etc.).
4. Usar a skill `decisao-linguagem` se houver dúvida entre destinos.
5. Delegar para o agente backend especializado do destino.
6. Delegar para `pe-agents-ibm-banco-dados` quando schema ou dados forem afetados.
7. Delegar para `pe-agents-ibm-sre` quando infra, container ou deploy forem afetados.
8. Acionar `pe-agents-ibm-tester` para validar cada fase migrada.

---

## Referências

- Martin Fowler — Strangler Fig Application
- Sam Newman — Monolith to Microservices
- AWS 5R Framework
- Microsoft .NET Upgrade Assistant docs
- OpenRewrite / Moderne documentation
- OWASP Top 10 for LLMs 2025 (quando migração envolver IA)
