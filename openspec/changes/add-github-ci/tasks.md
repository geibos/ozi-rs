## 1. Toolchain pinning

- [ ] 1.1 Создать `rust-toolchain.toml` в корне с `[toolchain] channel = "1.84.0"` (или текущая stable на момент мержа) и `components = ["clippy", "rustfmt"]`
- [ ] 1.2 Создать `.nvmrc` со строкой `lts/iron` (Node 20.x)
- [ ] 1.3 Проверить локально: `cargo --version`, `rustc --version`, `node --version` соответствуют пинам после `rustup show` / `nvm use`

## 2. Audit configuration

- [ ] 2.1 Создать `.cargo/audit.toml` с пустой секцией `[advisories] ignore = []` и комментарием-инструкцией, как добавлять ignore-записи с обоснованием
- [ ] 2.2 Запустить локально `cargo install cargo-audit --locked` и `cargo audit` — убедиться, что текущий `Cargo.lock` чист (или зафиксировать ignore с обоснованием)
- [ ] 2.3 Запустить локально `npm audit --omit=dev --audit-level=high` — убедиться, что нет high/critical advisories в прод-зависимостях

## 3. CI workflow (`.github/workflows/ci.yml`)

- [ ] 3.1 Добавить триггеры: `pull_request` (branches: main), `push` (branches: main), `workflow_dispatch`
- [ ] 3.2 Добавить `concurrency: { group: ci-${{ github.ref }}, cancel-in-progress: true }`
- [ ] 3.3 Добавить `permissions: { contents: read }` на уровне workflow
- [ ] 3.4 Job `lint-and-test` на `ubuntu-latest`:
  - [ ] 3.4.1 `actions/checkout@v4`
  - [ ] 3.4.2 Установка системных зависимостей Tauri (webkit2gtk-4.1, libsoup-3.0, librsvg2, libayatana-appindicator3, build-essential, etc.)
  - [ ] 3.4.3 `dtolnay/rust-toolchain@stable` с `toolchain` из `rust-toolchain.toml` (auto-pickup), компоненты `clippy`, `rustfmt`
  - [ ] 3.4.4 `Swatinem/rust-cache@v2` с `workspaces: src-tauri`
  - [ ] 3.4.5 `actions/setup-node@v4` c `node-version-file: '.nvmrc'`, `cache: 'npm'`
  - [ ] 3.4.6 `extractions/setup-just@v2` для запуска recipes
  - [ ] 3.4.7 `npm ci`
  - [ ] 3.4.8 Шаг `cargo fmt --all --manifest-path src-tauri/Cargo.toml -- --check`
  - [ ] 3.4.9 Шаг `just clippy`
  - [ ] 3.4.10 Шаг `just check-rust`
  - [ ] 3.4.11 Шаг `just test-rust`
  - [ ] 3.4.12 Шаг `just lint`
  - [ ] 3.4.13 Шаг `just check-ui`
  - [ ] 3.4.14 Шаг `just test-ui`
- [ ] 3.5 Job `openspec-validate` на `ubuntu-latest`:
  - [ ] 3.5.1 `actions/checkout@v4`
  - [ ] 3.5.2 Установить openspec CLI (`npm i -g @openspec/cli` или `npx openspec`, в зависимости от того, как он распространяется — уточнить в `package.json` / docs)
  - [ ] 3.5.3 Запустить `openspec list --json` и в цикле `openspec validate <change> --strict` для каждого активного изменения
- [ ] 3.6 Job `security-audit` на `ubuntu-latest`:
  - [ ] 3.6.1 `taiki-e/install-action@v2` для `cargo-audit`
  - [ ] 3.6.2 Шаг `cargo audit --deny warnings --file src-tauri/Cargo.lock`
  - [ ] 3.6.3 `actions/setup-node@v4` + `npm ci`
  - [ ] 3.6.4 Шаг `npm audit --omit=dev --audit-level=high`
- [ ] 3.7 Job `tauri-build-smoke` с матрицей `os: [ubuntu-latest, macos-latest, windows-latest]`:
  - [ ] 3.7.1 Установка системных зависимостей (Linux-only step через `if: matrix.os == 'ubuntu-latest'`)
  - [ ] 3.7.2 `dtolnay/rust-toolchain@stable`, `Swatinem/rust-cache@v2`, `setup-node@v4`, `npm ci`
  - [ ] 3.7.3 Шаг `npm run tauri build -- --no-bundle`
  - [ ] 3.7.4 `defaults.run.shell: bash` на уровне job, чтобы pwsh-цитирование не ломало команды
- [ ] 3.8 Убедиться, что все 4 job-а запускаются параллельно и не блокируют друг друга

## 4. Release workflow (`.github/workflows/release.yml`)

- [ ] 4.1 Триггер: `push.tags: ['v*']`
- [ ] 4.2 `permissions: { contents: write }`
- [ ] 4.3 Concurrency без cancel-in-progress: `group: release-${{ github.ref }}`
- [ ] 4.4 Job `release` с матрицей:
  - `{ platform: macos-latest, args: '--target universal-apple-darwin' }`
  - `{ platform: windows-latest, args: '' }`
- [ ] 4.5 Шаги job:
  - [ ] 4.5.1 `actions/checkout@v4`
  - [ ] 4.5.2 `dtolnay/rust-toolchain@stable` (с дополнительными targets для macOS universal: `aarch64-apple-darwin` + `x86_64-apple-darwin`)
  - [ ] 4.5.3 `Swatinem/rust-cache@v2`
  - [ ] 4.5.4 `actions/setup-node@v4` (`.nvmrc`), `npm ci`
  - [ ] 4.5.5 `tauri-apps/tauri-action@v0` с `tagName: ${{ github.ref_name }}`, `releaseName: ${{ github.ref_name }}`, `releaseDraft: true`, `args: ${{ matrix.args }}`
  - [ ] 4.5.6 `env: { GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }} }`

## 5. Документация

- [ ] 5.1 Создать `docs/ci.md`:
  - [ ] 5.1.1 Список всех CI-гейтов (Rust / frontend / OpenSpec / audit / smoke build)
  - [ ] 5.1.2 Триггер-матрица (PR / push / dispatch / tag)
  - [ ] 5.1.3 Релизный процесс: как создать тег `v*`, что попадёт в draft release, как опубликовать
  - [ ] 5.1.4 Процедура апдейта `rust-toolchain.toml` и `.nvmrc` (что проверить локально)
  - [ ] 5.1.5 Как добавить ignore в `.cargo/audit.toml` (формат + требование justification)
  - [ ] 5.1.6 Ссылка на текущие лимиты GitHub Actions / ожидаемое время прогона
- [ ] 5.2 Обновить `AGENTS.md` в разделе "Commands" / "Documentation":
  - [ ] 5.2.1 Добавить ссылку на `docs/ci.md`
  - [ ] 5.2.2 Упомянуть, что CI зовёт те же `just` recipes (локально и на runner идентично)
- [ ] 5.3 Обновить `CLAUDE.md` в разделе "Workflow": добавить пункт "перед PR прогнать `just clippy`, `just check`, `just lint`, `just test`, и убедиться, что `openspec validate --strict` проходит для активных изменений — иначе CI заблокирует мерж"
- [ ] 5.4 (Опционально) Добавить в `justfile` рецепт `ci: clippy check lint test` — алиас, повторяющий локально базовые гейты

## 6. Верификация

- [ ] 6.1 Запушить change в feature-ветку (`feature/add-github-ci`), открыть PR в `main` — убедиться, что все job-ы зелёные на cached-run < 15 мин
- [ ] 6.2 Проверить `workflow_dispatch` вручную из GitHub UI на feature-ветке
- [ ] 6.3 На временной ветке создать тестовый тег `v0.0.0-ci-test`, проверить, что `release.yml` собирает артефакты под macOS+Windows и создаёт draft release. После проверки — удалить тестовый release и тег
- [ ] 6.4 Намеренно сломать clippy (например, добавить `let _x: i32 = "hello";` в тестовом коммите) — убедиться, что CI падает на этом гейте. Откатить коммит
- [ ] 6.5 Намеренно сломать OpenSpec (добавить change без `tasks.md`) — убедиться, что `openspec-validate` job падает. Откатить
- [ ] 6.6 Записать в `docs/ci.md` инструкцию, как настроить branch protection rules в GitHub UI (required checks: lint-and-test, openspec-validate, security-audit, tauri-build-smoke / все три ОС)

## 7. Финализация

- [ ] 7.1 Прогнать `openspec validate add-github-ci --strict` локально
- [ ] 7.2 Проверить, что все файлы созданы и согласованы (`proposal.md`, `design.md`, `tasks.md`, `specs/ci-pipeline/spec.md`)
- [ ] 7.3 После мержа PR — выполнить `openspec archive add-github-ci`
