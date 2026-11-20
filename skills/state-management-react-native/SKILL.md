---
name: state-management-react-native
description: "Define ownership de estado em React Native separando. Use quando: escolher Zustand, Context."
user-invocable: true
---

# State Management React Native

## Quando Usar

- Ao escolher a estrategia de estado do app
- Ao encontrar stores inchadas, contexts grandes ou duplicacao de cache
- Ao revisar persistencia local, sessao e sincronizacao com backend

## Regras de Ownership

- Estado de tela temporario fica no componente ou em hook local
- Estado global de UI fica em store pequena e especializada
- Estado remoto fica em cache de servidor, preferencialmente TanStack Query
- Estado sensivel persistido usa storage seguro quando aplicavel
- Navegacao nao deve virar repositório de estado arbitrario

## Procedimento

### Passo 1 — Classificar cada dado

Para cada dado, responda:
- pertence so a esta tela?
- precisa sobreviver a relancamento do app?
- vem do backend e precisa cache/invalidation?
- contem segredo ou token?

### Passo 2 — Escolher o mecanismo certo

| Tipo de dado | Ferramenta preferida |
|---|---|
| Formulario local | `useState` ou `react-hook-form` |
| Estado global simples | `zustand` |
| Sessao / auth | store dedicada + storage seguro |
| Cache de API | `@tanstack/react-query` |
| Preferencias nao sensiveis | `AsyncStorage` ou `MMKV` |
| Dados sensiveis | `SecureStore` ou Keychain equivalente |

### Passo 3 — Evitar antipadroes

- Nao use Context gigante para tudo
- Nao duplique dados remotos em Zustand sem motivo
- Nao misture regras de negocio com adaptadores de storage
- Nao persista tokens em storage inseguro por conveniencia

### Passo 4 — Estruturar as stores

- Uma store por responsabilidade clara
- Actions explicitam side effects
- Seletores minimizam rerenders
- Persistencia so no que realmente precisa sobreviver

## Output Esperado

- Mapa claro de ownership de estado por modulo
- Recomendacao de stack de estado coerente
- Estrutura de stores, cache e persistencia definida
- Lista de antipadroes encontrados e correcoes sugeridas
