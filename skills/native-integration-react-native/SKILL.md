---
name: native-integration-react-native
description: "Planeja e implementa integracoes nativas em React. Use quando: trabalhar com permissoes, deep linking."
user-invocable: true
---

# Native Integration React Native

## Quando Usar

- Ao integrar recursos do dispositivo ou SDKs nativos
- Ao decidir entre Expo plugin, modulo existente ou implementacao customizada
- Ao tratar diferencas entre iOS e Android

## Procedimento

### Passo 1 — Identificar o boundary nativo

Defina claramente:
- qual capability do dispositivo ou SDK esta em jogo
- qual parte fica na camada React Native
- qual parte exige configuracao ou codigo por plataforma

### Passo 2 — Escolher estrategia

Prioridade recomendada:
1. API nativa do Expo, quando suficiente
2. Biblioteca React Native madura e mantida
3. Config plugin ou ajuste nativo pequeno
4. Modulo customizado nativo apenas quando necessario

### Passo 3 — Modelar contratos

- Isole a integracao atras de serviço/adaptador tipado
- Evite espalhar chamadas de permissao e SDK por telas
- Centralize tratamento de erro, fallback e feature detection

### Passo 4 — Tratar plataforma explicitamente

- Documente diferencas iOS/Android
- Mapeie permissoes, manifestos, plist, gradle e config plugins
- Defina comportamento quando capability nao estiver disponivel

## Output Esperado

- Boundary nativo definido com responsabilidades claras
- Estrategia escolhida entre Expo, biblioteca existente ou modulo customizado
- Lista de ajustes por plataforma e riscos de integracao
- Contrato tipado para consumo pelo app React Native
