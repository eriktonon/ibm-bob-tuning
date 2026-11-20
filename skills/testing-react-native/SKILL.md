---
name: testing-react-native
description: "Define e executa estrategia de testes em React Native. Use quando: configurar Jest, React Native."
user-invocable: true
---

# Testing React Native

## Quando Usar

- Ao criar a base de testes do projeto
- Ao adicionar cobertura para componentes, hooks, stores e fluxos mobile
- Ao revisar confiabilidade de autenticacao, onboarding, checkout ou outras jornadas criticas

## Piramide Recomendada

- Testes unitarios para funcoes puras, formatadores e casos de uso
- Testes de integracao para componentes, hooks e telas com React Native Testing Library
- E2E para fluxos criticos e poucas jornadas de negocio

## Procedimento

### Passo 1 — Configurar ferramentas

- Jest como runner base
- React Native Testing Library para renderizacao e interacao
- Detox ou Maestro para E2E, conforme a stack do projeto

### Passo 2 — Priorizar cobertura

Teste primeiro:
- autenticacao
- onboarding
- navegacao critica
- formularios complexos
- stores de sessao e persistencia
- integracoes nativas com fallback claro

### Passo 3 — Tratar mocks com disciplina

- Mock apenas fronteiras externas
- Nao mockar logica pura desnecessariamente
- Centralizar mocks de modulo nativo reutilizaveis

### Passo 4 — Validar ergonomia dos testes

- Prefira asserts por comportamento visivel
- Evite snapshots grandes e pouco informativos
- Garanta nomes de teste orientados a comportamento

## Output Esperado

- Estrategia de testes adequada ao app
- Setup de Jest e React Native Testing Library definido
- Plano de cobertura para fluxos criticos
- Recomendacao objetiva de E2E quando aplicavel
