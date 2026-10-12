Você está no repositório Kinematic. Implemente suporte completo a projetos Lua, um executável de editor e um fluxo de edição com atualização automática e incremental. Entregue código funcional, exemplos, documentação e validação; avance por etapas completas até atender ao escopo abaixo.

Este prompt foi preparado a partir da infraestrutura do commit a066c4e e das decisões do mantenedor. Reinspecione o código antes de modificar contratos: os caminhos citados orientam a análise, e o código atual é a autoridade. Leia AGENTS.md e src/ui/README.md. Preserve alterações locais que não pertençam a esta tarefa.

Atualização de escopo: remover integralmente os efeitos prontos da API Rust e Lua. Animações de entrada, saída e transformação devem ser construídas manualmente por propriedades e tweens.

As decisões de produto já estão tomadas:

| Decisão | Requisito |
|---|---|
| Produto | Executável que cria e abre projetos Lua, mantendo Kinematic utilizável como biblioteca Rust. |
| Compatibilidade | Mudanças na API Rust estão autorizadas quando simplificarem a integração. Atualize macros, exemplos, testes, documentação e todos os callers afetados; documente a migração. |
| Linguagem | Lua comum, sem exigir tipagem estática ou criar uma linguagem própria. Use inicialmente Lua 5.4 embutido via mlua, escolhendo versões compatíveis durante a implementação. |
| Atualização automática | Após uma pequena pausa ao digitar, atualizar a prévia a partir dos buffers em memória, sem salvar automaticamente. Detectar também alterações salvas por editores externos. |
| UI | Integrar os painéis do editor: código, arquivos, diagnósticos, Inspector, Scene Tree, Timeline, preview e configurações. Painéis escritos em Lua e um toolkit de UI para cenas não fazem parte desta entrega. |
| Identidade | Aceitar IDs explícitos para os objetos que precisam preservar identidade entre edições. Objetos sem ID continuam permitidos, com garantias de preservação menores e documentadas. |
| Simulações | Quando lógica ou entradas mudarem, recalcular apenas as simulações afetadas até o tempo atual. Preservar histórico quando mudanças forem exclusivamente visuais e não afetarem suas dependências. |
| Inspector | Persistir overrides no projeto, indicar sua existência e permitir restaurar o valor definido pelo script. |
| Assistência de código | Integrar Lua Language Server para autocomplete, documentação, assinatura de funções e navegação para definições, além de abas, busca, undo/redo e diagnósticos. |

Não peça novamente aprovação para essas decisões nem para refatorações internas necessárias. Resolva detalhes rotineiros conforme as convenções do projeto. Caso apareça uma ambiguidade que altere o comportamento do produto, explique o caso concreto e prossiga com o trabalho independente.

A infraestrutura existente deve ser reutilizada:

| Área | Ponto de partida e adaptação necessária |
|---|---|
| Aplicação | src/app.rs mantém SDL, OpenGL, Skia, ImGui e o loop. Hoje exige um Project antes de executar; precisa admitir a tela inicial e a troca de projeto. |
| Projeto | src/core/project.rs usa nomes estáticos, SceneFactory como ponteiro de função e configurações relativas ao diretório atual. Generalizar o carregamento e introduzir uma raiz explícita de projeto. |
| Construção e avaliação | SceneBuilder, Task, Animator e Scene::update(time), em src/core. A linguagem deve usar a mesma semântica de animação e avaliação. |
| Metadados | TrackInfo, TrackableInfo, TrackTarget, TrackValue, Inspection e macros em packages/kinematic-macros/src. Atualmente dependem de tipos Rust e metadados estáticos. |
| Simulação | src/core/components/simulation.rs implementa passos fixos, operações agendadas e checkpoints clonados. SimulationState exige Clone + Send + Sync. |
| Signals | src/core/signal.rs implementa callbacks temporais com overrides restaurados na avaliação seguinte. |
| Editor | src/editor/editor.rs mantém cenas, seleção, tempo, overrides temporários e caches de avaliação/renderização. set_event_duration atualmente substitui a cena e limpa a seleção. |
| UI | src/ui/mod.rs orquestra os painéis; src/ui/workspace.rs define o docking; cada painel possui seu estado. |
| Reload atual | src/reload.rs executa cargo build e reinicia o processo. Isso pode continuar atendendo o desenvolvimento nativo, mas não é o reload de projetos Lua. |
| Desenho | Draw2D, Draw3D, RenderContext3D, src/core/objects/render.rs e src/renderer/canvases.rs. Preservar a integração com bounds, seleção, canvases e shaders. |
| Recursos | GeometryKey, caches de geometria/programas/texturas e CanvasTexture. Há identidades associadas à cena e à entidade que precisam continuar coerentes durante reload. |
| Exportação | src/renderer reutiliza os recursos gráficos existentes e envia frames RGBA para FFmpeg. Manter esse caminho para Lua. |

O core deve continuar independente da UI e da posse do loop gráfico. Mantenha o conhecimento de Lua em um módulo de scripting bem delimitado; generalize apenas os contratos realmente necessários para runtime, propriedades, callbacks e identidades. Não introduza um segundo ECS, outro avaliador de animações ou um sistema paralelo de propriedades para os scripts. Não converta o projeto inteiro para Arc/Mutex para acomodar a VM e não use unsafe Send, transmute de lifetimes ou vazamentos de memória para fabricar referências estáticas.

Use os metadados gerados pelas macros como fonte dos bindings nativos, descrições do Inspector e informações da API para documentação/completion. Funções especiais podem ter adapters específicos, mas não duplique à mão todos os campos dos objetos e seus limites. Suporte propriedades dinâmicas no mesmo caminho de tracks, interpolação, inspeção, snapshots. Evite criar um framework genérico de reflexão além do necessário para esse comportamento.

O executável deve permitir criar, abrir, fechar e trocar projetos sem recompilar Rust e sem exigir uma instalação separada de Lua. Reaproveite o suporte a RON para um manifesto único, gravável pelo editor, contendo versão do formato, nome, cenas ordenadas, resolução, FPS e margens. Scripts, assets, overrides e caches devem ter responsabilidades distintas; não mantenha duas fontes de configuração concorrentes em Lua e RON.

Todos os caminhos de cenas, módulos, recursos, caches e exportações devem resolver a partir da raiz do projeto, sem mudar globalmente o diretório do processo. A biblioteca Rust também deve conseguir fornecer esse contexto de maneira explícita. Preserve os formatos existentes quando viável e forneça migração clara quando necessário.

Inclua projetos iniciais pequenos para 2D e 3D, criação de scripts e uma árvore de arquivos que permita abrir, criar e renomear arquivos e pastas. Não implemente renomeação como refatoração silenciosa de imports arbitrários. Fechar documentos, trocar projeto ou sair com alterações não salvas deve oferecer salvar, descartar ou cancelar. Arquivos inválidos nunca devem ser substituídos automaticamente por defaults. Salvar arquivos deve evitar truncamento e detectar conflitos com versões externas.

Integre o editor de código à infraestrutura Dear ImGui existente. Avalie dear-imgui-cte como primeira opção, verificando sua compatibilidade real. O repositório analisado usa dear-imgui-rs, dear-imgui-sdl3 e dear-imgui-glow 0.15.1; o CTE consultado pertence a uma linha posterior e é apresentado como preview. Se necessário, atualize o conjunto compatível e adapte os backends e texturas conforme a documentação, sem misturar ABIs ou criar um segundo contexto ImGui. Não substitua o widget por um editor de texto artesanal.

O editor embutido deve oferecer:

- Abas com estado independente, indicação de alterações não salvas, números de linha, destaque Lua, indentação, seleção, clipboard, undo/redo, busca/substituição e navegação para arquivo/linha.
- Fonte monoespaçada, Unicode correto e integração com o tema e escala atuais.
- Painel de diagnósticos com origem, arquivo, linha, coluna e traceback quando disponível; clicar deve abrir o documento e posicionar o cursor.
- Console de print e erros com limites de retenção e agregação de erros repetidos, evitando uma mensagem idêntica por frame.
- Preservação de cursor, seleção, scroll, histórico de edição e documentos abertos durante reload.
- Layout dockável que permita trabalhar com código e preview simultaneamente, respeitando os estados dos painéis e a seleção compartilhada pelo Editor.
- Roteamento de atalhos por foco. Espaço, setas, B, F e Esc usados pelo editor de texto não podem acionar playback, rebuild, fullscreen ou saída do aplicativo. Corrija o tratamento global de Esc em src/app.rs.
- Estados de carregamento, aplicação pendente e falha legíveis, mantendo a última prévia válida acessível.

O documento em memória é a fonte do texto editado. Salvamento e reload são ações distintas. Use debounce curto, inicialmente próximo de 400 ms, ajustável se as medições justificarem. Salvar pode solicitar atualização imediata. Arquivos externos devem ser observados com tratamento de saves por rename/substituição atômica, eventos duplicados e arquivos temporariamente incompletos. Se houver um buffer local modificado e uma alteração externa, apresente o conflito; não sobrescreva nenhum lado silenciosamente. Filtre arquivos gerados, caches, output, target e .git; reconheça gravações do próprio editor para evitar ciclos de reload.

Integre LuaLS como processo separado por projeto, com ciclo de vida e transporte assíncronos. Sincronize documentos e versões via LSP, incluindo textos ainda não salvos. Descarte respostas e diagnósticos de versões antigas; trate corretamente posições UTF-8/UTF-16 conforme a negociação do protocolo e o widget. Gere as definições Lua da API a partir dos metadados registrados, incluindo objetos, propriedades, tipos de valor, callbacks e recursos. Configure o workspace sem sobrescrever configurações do usuário. Não exija anotações de tipos para executar scripts nem transforme avisos de tipos em bloqueio de execução.

Defina uma forma documentada de localizar ou distribuir LuaLS; não faça downloads silenciosos. A ausência ou falha do servidor não pode impedir edição básica, execução ou diagnóstico de runtime. Ainda assim, a entrega deve testar a integração real de autocomplete e navegação com o servidor disponível.

A API Lua deve cobrir as funcionalidades públicas do Kinematic. Antes de implementar os bindings, faça um inventário conciso do que src/lib.rs, src/prelude.rs e os módulos públicos expõem. Use esse inventário para não omitir objetos ou funcionalidades menos visíveis:

| Família | Cobertura necessária |
|---|---|
| Valores | Vetores 2D/3D, cores, quaternions, retângulos/quad, matrizes necessárias ao desenho, enums e operações matemáticas apropriadas. |
| Objetos 2D | Formas, linhas, imagens, texto, LaTeX, grupos, containers, canvases e projeções existentes. |
| Objetos 3D | Formas, linhas, imagens, texto, LaTeX, grupos, containers, luzes, canvases e projeções existentes. |
| Hierarquia | Root, anexação, ordenação, consultas, transforms locais/globais, anchors, bounds e lifetimes. |
| Animação | Propriedades e subcanais, tweens absolutos/relativos, from/to/by, easing, duration, delay, tarefas compostas e tracks independentes. |
| Agendamento | play, wait, event, chain, all, parallel e ciclos de repeat, respeitando suas diferenças reais. |
| Estado de objetos | Snapshots, save/restore/reset e consultas públicas relevantes. |
| Signals | Registro em objetos/tracks, tempo e índice de frame, início/fim e stop agendado. |
| Simulações | Estado inicial, update automático/manual, checkpoint, leitura de entradas no tempo do passo, mutações agendadas e random determinístico. |
| Recursos gráficos | Materiais, câmeras, filtros, shaders 2D/3D, uniforms animáveis, texturas de canvases e dependências entre canvases. |
| Extensão | Componentes e objetos definidos em Lua, com desenho e simulação próprios; registro de extensões nativas Rust quando uma capacidade externa realmente precisar de Rust. |

Paridade significa acesso às funcionalidades do Kinematic, inclusive desenho personalizado. Os reexports de hecs, Skia e three-d não exigem reproduzir integralmente as APIs dessas dependências em Lua. Forneça uma API de desenho suficientemente completa para criar geometrias e visuais próprios, em vez de limitar Lua aos objetos prontos.

Adote uma convenção coerente de handles para propriedades: obj.position:get(), obj.position:set(value) e obj.position:to(value, duration, easing). Os campos trackable devem continuar diretamente no objeto, sem handlers intermediários de componente. Get devolve um valor; set altera o valor atual sem criar animação; to prepara um tween para play. Os significados devem ser os mesmos para propriedades nativas, dinâmicas e uniforms, com validação e diagnósticos adequados.

Use este exemplo como referência de ergonomia, ajustando nomes somente quando necessário para manter coerência com a implementação:

~~~lua
local k = require("kinematic")

return k.scene("Intro", function(s)
    local ball = s:circle {
        id = "ball",
        radius = 24,
        fill = k.color("#60A5FA"),
        position = k.vec2(-300, 0),
    }

    s:world_2d():add(ball)

    s:all(function()
        s:play(ball.position:to(k.vec2(300, 0), 2.0, k.ease.cubic_out))
        s:play(ball.radius:to(48, 1.0))
    end)

    s:event("Apresentação")

    local center = ball.position:get()
    local motion = ball:signal(function(obj, frame)
        obj.position:set(center + k.vec2(0, math.sin(frame.time * 3) * 40))
    end)

    s:wait(3.0)
    motion:stop()
    s:play(ball.opacity:to(0, 0.5))
    ball:remove()
end)
~~~

O corpo da cena constrói um agendamento; wait não dorme e play não abre um segundo loop. all executa animações simultaneamente e avança pela maior duração. parallel agenda trabalho sem avançar ou estender a timeline externa. O repeat nativo compila um ciclo finito uma vez, com suas restrições de criação e estrutura; use um nome Lua válido, como loop, porque repeat é palavra reservada. Um for Lua representa repetição finita na construção.

Signals do Kinematic são callbacks de avaliação temporal, e event é um trecho temporal nomeado. Não altere seus significados para um sistema de mensagens emit/connect. Preserve a restauração dos overrides dos signals entre avaliações e as restrições de mutação da timeline durante callbacks.

A criação de objetos deve preservar o contrato de nodes inicialmente inativos e a ativação ao anexá-los à hierarquia no tempo de construção. remove encerra o lifetime, permitindo seek para antes da remoção. Excluir uma definição do código durante reload é outra operação: retire suas contribuições da nova revisão e invalide referências que não sejam mais válidas. Não confunda essas duas formas de remoção.

Defina componentes Lua com propriedades declaradas, defaults, limites e escolhas, e objetos Lua que componham esses componentes com os componentes espaciais nativos. Campos personalizados devem aparecer no Inspector, Timeline, snapshots. Variáveis Lua continuam dinâmicas; propriedades animáveis têm tipos conhecidos em runtime para interpolação e validação. Trate campos duplicados, tipos incompatíveis, valores não finitos, ranges, enums e mudança de schema com erros claros.

Handles devem validar identidade da cena, existência/generation da entidade e versão do recurso ou definição quando aplicável. Não exponha ponteiros, referências Rust livres ou IDs de entidade convertidos em números Lua potencialmente imprecisos. GC de um handle não remove um objeto da timeline. Evite ciclos de retenção entre VM, callbacks, scene world e recursos; descarte registros de callbacks substituídos e recursos obsoletos após perderem seus usuários.

O reload deve preservar o mundo ativo e os recursos compatíveis. Reexecutar a descrição Lua de uma cena dependente pode ser necessário; isso não autoriza reconstruir o ECS inteiro ou todas as simulações a cada tecla. Implemente reconciliação das definições com o estado existente.

A construção candidata deve partir dos defaults, valores declarados e overrides da revisão candidata, nunca dos valores que o playback já avaliou no mundo ativo. Leituras feitas durante a construção precisam enxergar o estado de construção correspondente ao agendamento, preservando a semântica dos builders atuais. Aplicar repetidamente a mesma revisão não pode acumular deslocamentos, multiplicar escalas, consumir novamente o random ativo ou duplicar operações.

O fluxo esperado é:

1. Capturar uma revisão coerente das fontes: buffers abertos têm precedência sobre o disco. Cada solicitação tem uma versão; resultados ultrapassados não podem ser aplicados.
2. Detectar módulos e recursos alterados e seus dependentes. Rastrear require e dependências carregadas pela API de recursos do projeto.
3. Preparar as definições candidatas em um ambiente separado do estado ativo, sem mutar o ECS em execução, os overrides persistidos ou os arquivos de eventos.
4. Reavaliar apenas as descrições de cenas dependentes e produzir dados comparáveis sobre objetos, propriedades, hierarquia, tracks, callbacks, simulações e recursos.
5. Validar IDs, schemas, referências, agendamento e recursos necessários. Produzir um conjunto explícito de alterações e invalidações.
6. Aplicar as mudanças compatíveis em um ponto seguro entre avaliações/frames. Preservar entidades, handles, seleção, câmeras do editor, documentos e recursos não afetados.
7. Reavaliar no mesmo tempo local da cena ativa; se a duração encolher, limitar o tempo ao intervalo válido. Recalcular offsets das cenas posteriores quando necessário.
8. Em falha de preparação, preservar integralmente a revisão ativa e apresentar o erro. A alteração candidata só se torna ativa quando sua aplicação e primeira avaliação exigida forem válidas.

A atomicidade se aplica ao estado controlado pela engine. Mantenha efeitos externos fora da preparação de definições: não prometa desfazer I/O arbitrário de scripts. Evite que uma avaliação candidata modifique globals, tabelas compartilhadas ou package.loaded utilizados pela revisão ativa.

Não use o endereço de uma função Lua, o RegistryKey ou a versão do arquivo inteiro como única evidência de alteração de um callback. Uma edição no draw não deve invalidar automaticamente um update inalterado no mesmo módulo. Considere corpo relevante, capturas e dependências, desconsiderando diferenças puramente de localização/comentários quando possível. Não tente provar equivalência geral de programas Lua; quando não for possível demonstrar compatibilidade, invalide o menor conjunto seguro e registre o motivo.

Os objetos com ID explícito devem ser reconciliados por uma chave estável com escopo documentado, preferencialmente projeto/cena/ID. Não derive identidade persistente de linha, nome de variável ou ordem global de criação. Detecte IDs duplicados antes de alterar a cena ativa. Para objetos sem ID, use um fallback local documentado, aceitando substituição quando a correspondência for ambígua. Não transfira estado de um objeto para outro apenas porque ocupam a mesma posição numa lista.

O comportamento mínimo verificável do reload é:

| Mudança | Efeito permitido |
|---|---|
| Comentários/formatação | Atualizar posições de diagnóstico; preservar entidades, recursos, estado e checkpoints. |
| Cor ou outra propriedade apenas visual | Atualizar propriedade e caches visuais dependentes, sem reiniciar simulações independentes. |
| Raio/geometria | Atualizar bounds e geometria afetados; recalcular simulação apenas se essa propriedade for uma entrada dela. |
| Tween/easing/duração | Substituir a contribuição de tracks afetada, sem duplicar keyframes antigos; propagar mudanças de tempo quando necessário. |
| Evento temporal | Atualizar o agendamento dependente sem substituir obrigatoriamente a cena inteira. |
| Draw/bounds | Atualizar callbacks e caches visuais dependentes; manter estado de simulação compatível. |
| Update/estado inicial/seed/entradas de simulação | Invalidar checkpoints incompatíveis e recalcular o conjunto afetado até o tempo atual. |
| Novo objeto/objeto retirado/reordenação | Aplicar a diferença estrutural mantendo os demais IDs e seleções. |
| Mudança incompatível de schema/tipo | Recriar os dados ou o objeto afetado, com diagnóstico de incompatibilidade; preservar o restante. |
| Imagem, shader ou mesh | Atualizar o recurso e seus dependentes; preservar a versão válida anterior se o novo recurso falhar. |
| Módulo compartilhado | Atualizar suas cenas/callbacks dependentes sem recarregar indiscriminadamente o projeto inteiro. |

Tracks já compiladas precisam poder ser substituídas sem acumular contribuições de revisões antigas. Preserve a ordenação temporal de keyframes e a avaliação correta dos valores iniciais/finais. Mudanças visuais devem invalidar somente os recursos gráficos e dependências afetados.

Reutilize as revisões existentes da cena e do plano de renderização, refinando-as quando necessário. Separe alteração visual, estrutura, programação temporal, inputs de simulação e conteúdo de recursos apenas onde essa distinção evita trabalho real. Preserve caches de avaliação sem permitir que uma mudança aplicada no mesmo instante seja ignorada.

A simulação Lua deve integrar-se ao scheduler existente, com estado de dados separado dos callbacks. Defina um formato de snapshot com cópia profunda para os valores suportados: escalares, tabelas, vetores, cores, quaternions e estado do Random da engine, conforme necessário. Não copie referências da VM como se fossem snapshots independentes. Rejeite valores não suportados com o caminho do campo; documente a política para ciclos e referências compartilhadas.

Cada instância precisa ter estado inicial independente. Passos fixos, updates manuais, checkpoints e mutações agendadas devem reproduzir o mesmo resultado em playback, seek e exportação. Estado que afeta o resultado não pode ficar oculto em globals ou upvalues mutáveis fora do mecanismo de snapshot. Recursos imutáveis podem ser referenciados por handles controlados.

As propriedades lidas por update devem ser amostradas no tempo daquele passo, e não no tempo final solicitado pelo editor. Identifique as entradas da simulação, preferindo declarações explícitas quando necessárias para distinguir propriedades visuais. Apenas registrar leituras de uma execução anterior não prova ausência de dependências em outros caminhos condicionais. Na falta de informação suficiente, invalide conservadoramente a simulação afetada.

Mudanças na lógica invalidam o histórico desde o início da instância, salvo quando existir uma prova de compatibilidade mais restrita. Mudanças numa entrada temporal podem preservar checkpoints anteriores ao primeiro instante afetado. FPS, lifetime, operações agendadas, seed e dependências também participam da validade. Recalcule dependentes quando houver simulações que leem outras; valide ciclos e defina a ordem.

O replay necessário deve manter a UI responsiva, com progresso/cancelamento quando custoso. Não atualize visivelmente a cena com estado parcialmente recalculado. Preserve o tempo do cursor e o estado de playback quando possível; não continue a lógica nova em cima do estado antigo de forma silenciosa.

Integre callbacks sem conflitos de borrow: Scene::update atualmente mantém empréstimos do world durante simulação, enquanto signals são avaliados depois de liberá-los. Ofereça views/contextos limitados aos dados corretos da fase e aplique mutações autorizadas em pontos seguros. Não entregue referências ao world que possam ser guardadas em Lua ou reentrar num borrow ativo.

Mantenha a VM e as chamadas que dependem do renderer no thread que possui esses recursos. Componentes hecs devem armazenar dados e identificadores compatíveis com seus requisitos de Send/Sync, sem embutir referências à VM ou closures Lua incompatíveis. Adapte o despacho de callbacks para o scheduler comum conseguir invocar scripts com segurança. Watchers e transporte do LuaLS podem trabalhar fora desse thread e entregar mensagens versionadas.

Todas as chamadas Lua devem propagar erros com traceback, arquivo, objeto e fase. Restabeleça escopos do scheduler, stacks de desenho e estados temporários mesmo quando houver erro. Um script inválido não pode fechar o editor via panic. Use limites de execução e memória apropriados para que loops acidentais não bloqueiem o editor indefinidamente; documente que hooks da VM não interrompem automaticamente uma operação Rust/GPU longa e limite essas operações quando necessário.

Uma falha encontrada ao avaliar ou desenhar depois de um reload válido deve interromper a operação afetada, manter a UI e o último frame válido disponíveis e permitir correção. Não exponha estado parcialmente escrito por um callback que falhou. Diferencie isso da falha de preparação de reload, na qual a revisão anterior continua ativa.

O desenho Lua precisa entrar no renderer existente e funcionar tanto em objetos personalizados quanto em simulações:

- Em 2D: paths com segmentos e curvas Bézier, fechamento, fill/stroke, cores/gradientes, transforms, save/restore, clipping e composição de camadas, além de primitivas, texto, imagens e acesso aos recursos necessários aos helpers existentes.
- Em 3D: criação e atualização de meshes com posições, índices, normais, UVs e cores quando suportadas; transforms, materiais, luzes, câmeras, texturas e shaders. Ofereça submissão em lote ou instancing onde necessário para não exigir milhares de transições Lua/Rust por frame.
- Os callbacks recebem coordenadas locais. O renderer continua responsável por hierarquia, transforms globais, visibilidade, clipping e composição de opacidade conforme seus contratos.
- Contextos de desenho só são válidos durante o callback. Use mecanismos seguros de escopo, como os oferecidos por mlua, e invalide userdata escapado. Recursos persistentes são handles distintos do contexto temporário.
- Bounds lógicos e visuais precisam participar de seleção, anchors, clipping e exportação. Inclua bounds de geometria deslocada e padding de shaders quando aplicável.
- Um draw não pode alterar o estado de simulação ou animação. Caches puramente gráficos pertencem ao sistema de recursos, com invalidação definida.
- Recursos de GPU permanecem no lado Rust e no contexto gráfico existente. Generalize GeometryKey para identidades dinâmicas sem colisões entre objetos/projetos, respeitando versões do conteúdo e liberação após substituição.
- Mudanças de uniforms não devem recompilar shaders; mudanças de transform/material não devem reconstruir meshes. Evite upload de geometria inalterada e reconstrução de buffers a cada frame.
- Preserve canvases/projeções 2D/3D, dependências entre canvases, filtros, materiais e o caminho de shaders nativos. Valide ciclos de recursos e referências a cenas incompatíveis.

Não limite a comprovação do desenho 3D a criar uma esfera nativa. Um exemplo Lua deve fornecer vértices/índices próprios e desenhá-los com material e shader. Um exemplo 2D deve construir paths próprios com limites não centrados e funcionar com animações manuais de propriedades.

Os overrides do Inspector devem ser dados persistentes do projeto, identificados por cena, ID explícito do objeto e chave estável da propriedade, nunca por TypeId serializado, número de linha ou endereço da entidade. Mostre o valor do script, o override e a ação de restaurar.

Adote overrides de configuração/valor-base: eles entram antes da construção das animações que dependem desses valores. Tracks e signals continuam avaliando normalmente. O Inspector deve distinguir valor-base e valor avaliado no tempo atual; editar a base não deve reescrever silenciosamente keyframes ou congelar uma propriedade animada. Use a mesma reconciliação para alterações do Inspector, incluindo invalidação de simulações quando suas entradas mudarem.

Objetos sem ID podem ter alterações temporárias claramente identificadas; persistência exige identidade explícita. Overrides incompatíveis ou órfãos após uma alteração do script devem ser apresentados para resolução, sem aplicá-los a outro objeto ou destruí-los silenciosamente. A ação de restaurar remove o override persistido. Durações de eventos continuam sendo dados editáveis da Timeline e participam da reconciliação.

Reload, salvamento e exportação precisam ter revisão definida:

- A exportação usa a revisão válida ativa da prévia, incluindo textos não salvos que já tenham sido aplicados. A UI deve indicar quando os buffers atuais têm erros e não correspondem à revisão ativa.
- Ao iniciar a exportação, fixe fontes, callbacks, overrides, seeds e conteúdo/versionamento dos recursos usados. Não permita misturar revisões entre frames.
- Edições podem continuar nos documentos, mas a aplicação de reload ao runtime exportado fica pendente até terminar ou cancelar. Recursos observados no disco também não podem mudar silenciosamente durante a exportação.
- Reutilize SDL/OpenGL/Skia/three-d/framebuffers existentes, avance exatamente 1/fps e escreva output/<nome do projeto>.mp4 dentro da raiz do projeto.
- Overlays e painéis do editor não entram nos frames. Erros de script/desenho durante exportação devem interromper com diagnóstico e seguir o tratamento de arquivo incompleto do exporter.
- Troca/fechamento de projeto deve liberar watchers, LuaLS, callbacks, VM, caches e recursos que não tenham mais usuários, na ordem adequada ao contexto gráfico.

Valide o comportamento com testes proporcionais e exemplos executáveis. Use os testes e helpers existentes, especialmente tests/custom_simulation.rs, tests/custom_draw_3d.rs, tests/custom_container.rs, tests/shader_uniforms.rs e examples/game_of_life.rs. Acrescente testes que exercitem o uso público e falhas reais, sem espelhar apenas a implementação interna.

Os critérios de aceitação incluem:

| Caso | Evidência esperada |
|---|---|
| Projeto | Criar, abrir, salvar e trocar dois projetos em raízes diferentes, sem recompilar ou cruzar arquivos/caches. |
| Biblioteca | Exemplos Rust atualizados continuam construindo e executando; mudanças públicas possuem orientação de migração. |
| Paridade | Exemplos Lua exercitam objetos 2D/3D, tracks, grupos, eventos, ciclos, snapshots, signals, canvases, projeções e shaders. |
| Componentes | Propriedade personalizada aparece no Inspector e Timeline, aceita animação e preserva limites/tipo em runtime. |
| Simulation | Game of Life e outro estado acumulativo fornecem resultados equivalentes após playback, seek para trás, seek direto e replay após reload. |
| Mudança visual | Alterar cor e draw preserva entidade, seleção e checkpoints de simulações sem dependência visual. |
| Mudança de lógica | Alterar update recalcula apenas instâncias/dependentes afetados e chega ao resultado de uma execução limpa da mesma revisão. |
| Reconciliação | Inserir um objeto antes de outro e reordenar declarações não troca a identidade dos objetos com ID. |
| Idempotência | Reaplicar uma revisão preserva os resultados sem acumular transforms, consumo de random ou operações. |
| Transação | Falha após preparar parte das alterações preserva entidades, propriedades, callbacks e arquivos persistidos da revisão ativa. |
| Timeline | Alterar evento ou tween não duplica tracks/callbacks nem deixa keyframes da revisão anterior. |
| Comentários | Editar comentários/espaços não reinicia simulações nem recria entidades/recursos. |
| Módulos | Alterar módulo compartilhado atualiza os dependentes corretos; import inexistente mantém a revisão ativa. |
| Erros | Sintaxe inválida, erro em callback, referência inválida e shader inválido geram diagnósticos e preservam o editor utilizável. |
| Recursos | Mesh personalizado 3D e paths 2D funcionam com bounds, picking, transforms, opacidade e animações de propriedades; recursos inalterados são reutilizados. |
| Arquivos | Reload durante digitação não salva; alteração externa com buffer local modificado apresenta conflito; configuração inválida não é sobrescrita. |
| Inspector | Overrides persistem após reabrir, aplicam-se ao ID correto e podem ser removidos; órfãos/incompatíveis ficam identificáveis. |
| LuaLS | Completion conhece a API e campos do objeto, hover mostra assinaturas e ir para definição funciona entre arquivos, inclusive com Unicode. |
| Foco | Digitar espaços, setas, B, F e Esc no editor não aciona controles globais; undo/redo permanece no documento correto. |
| Exportação | Reload pendente não muda a revisão exportada; frames correspondem à avaliação fixa da mesma revisão e não incluem overlays. |
| Ciclo de vida | Sequências repetidas de reload e troca de projeto não acumulam callbacks, registros de VM ou recursos GPU sem usuários. |
| Desempenho | Medições distinguem reavaliação Lua, aplicação de diferenças, replay, uploads e renderização; mudar uma cor não executa trabalho global injustificado. |

Para checks automatizados sem GPU, use o caminho raster e testes de core onde possível. Exercite os cenários gráficos no ambiente disponível e informe limitações reais de validação; não substitua um teste de renderização por um teste que apenas verifica a existência de um símbolo. Documente a plataforma efetivamente testada.

Execute cargo fmt --check, cargo check --workspace e cargo test --workspace. Garanta também a compilação dos exemplos e do novo executável; inclua cargo check --workspace --all-targets quando necessário para cobrir esses targets. Faça smoke tests do editor, de um projeto Lua 2D/3D, do reload e da exportação com FFmpeg. Se algum check depender de infraestrutura indisponível, registre exatamente o que faltou e conclua os checks independentes.

Organize a implementação em etapas integradas, mantendo o projeto compilável:

1. Ajustar os contratos mínimos de projeto, propriedades, runtime e identidades; estabelecer o inventário de paridade e a representação candidata necessária à reconciliação antes de acoplar os bindings a mutações do mundo ativo.
2. Entregar carregamento Lua e uma cena real com animação, signal, diagnóstico e objetos nativos.
3. Completar componentes, simulações e desenho personalizado 2D/3D no mesmo pipeline.
4. Implementar a reconciliação, dependências, overrides, replay seletivo e reload de recursos.
5. Integrar gestão de projetos, documentos, widget de código, LuaLS e atualização por buffers/arquivos.
6. Completar a matriz de funcionalidades, exemplos, exportação estável, testes, medições e documentação.

Essas etapas organizam a entrega; não são autorização para encerrar com apenas uma prova de conceito. Não declare suporte completo com TODOs nos caminhos exigidos, nem deixe o reload incremental como trabalho futuro. Também não acrescente debugger com breakpoints, linguagem própria, gerenciador de pacotes ou novos backends gráficos: não são necessários ao escopo definido.

Na entrega, resuma o que funciona, como abrir/criar um projeto, quais APIs Rust mudaram, quais checks passaram e quais limitações concretas restam. Inclua documentação da API Lua, exemplos completos e uma explicação curta das regras de identidade, overrides e invalidação. Não deixe apenas exemplos ilustrativos que não executam.

Referências técnicas para verificar durante a implementação:

- [mlua](https://docs.rs/mlua/latest/mlua/) e [escopos, hooks e limites da VM](https://docs.rs/mlua/latest/mlua/struct.Lua.html).
- [dear-imgui-rs e compatibilidade das extensões](https://github.com/Latias94/dear-imgui-rs#extensions).
- [dear-imgui-cte](https://docs.rs/dear-imgui-cte/latest/dear_imgui_cte/).
- [Lua Language Server](https://luals.github.io/) e [configuração de bibliotecas e workspace](https://luals.github.io/wiki/settings/).
- [notify e diferenças entre eventos de editores](https://docs.rs/notify/latest/notify/).
