## 1. Toolchain pinning

- [x] 1.1 Создать `rust-toolchain.toml` в корне с `[toolchain] channel = "1.95.0"` (актуальная stable на 2026-05) и `components = ["clippy", "rustfmt"]`, profile = minimal
- [x] 1.2 Создать `.nvmrc` со строкой `22` (актуальный LTS — `lts/jod`; в исходном плане был `lts/iron`/Node 20, но на 2026-05 он уже в maintenance)
- [x] 1.3 Проверить локально: `cargo --version` (1.95.0) и `node --version` (v25.x — совместимо с pin Node 22 LTS в CI)

## 2. Audit configuration

- [x] 2.1 Создать `.cargo/audit.toml` с секцией `[advisories]` и инструкцией, как добавлять ignore-записи с обоснованием
- [x] 2.2 Запустить локально `cargo audit --file src-tauri/Cargo.lock`. Зафиксировать ignore для трёх vulnerabilities `rustls-webpki 0.103.10` (RUSTSEC-2026-0098/0099/0104), приходящих транзитивно через Tauri 2.10.3 → reqwest → quinn. Recheck на каждом Tauri bump. CI запускает `cargo audit` без `--deny warnings`: 21 unmaintained-warning (gtk-rs / proc-macro-error / unic-*) остаются информационными, потому что фиксятся только на стороне upstream Tauri
- [x] 2.3 Запустить локально `npm audit --omit=dev --audit-level=high` — найден 1 moderate advisory (`protocol-buffers-schema` < 3.6.1 через maplibre-gl), что ниже порога high. EXIT=0, gate не падает

## 3. CI workflow (`.github/workflows/ci.yml`)

- [x] 3.1 Добавить триггеры: `pull_request` (branches: main), `push` (branches: main), `workflow_dispatch`
- [x] 3.2 Добавить `concurrency: { group: ci-${{ github.workflow }}-${{ github.ref }}, cancel-in-progress: true }`
- [x] 3.3 Добавить `permissions: { contents: read }` на уровне workflow
- [x] 3.4 Job `lint-and-test` на `ubuntu-latest`:
  - [x] 3.4.1 `actions/checkout@v4`
  - [x] 3.4.2 Установка системных зависимостей Tauri (webkit2gtk-4.1, libsoup-3.0, libjavascriptcoregtk-4.1, librsvg2, libayatana-appindicator3, libssl-dev, build-essential, curl, wget, file)
  - [x] 3.4.3 `dtolnay/rust-toolchain@master` с `toolchain: stable` (подхватывает версию из `rust-toolchain.toml`), компоненты `clippy`, `rustfmt`
  - [x] 3.4.4 `Swatinem/rust-cache@v2` с `workspaces: src-tauri`
  - [x] 3.4.5 `actions/setup-node@v4` c `node-version-file: .nvmrc`, `cache: npm`, `cache-dependency-path: package-lock.json`
  - [x] 3.4.6 `extractions/setup-just@v2` для запуска recipes
  - [x] 3.4.7 `npm ci`
  - [x] 3.4.8 Шаг `cargo fmt --all --manifest-path src-tauri/Cargo.toml -- --check`
  - [x] 3.4.9 Шаг `just clippy`
  - [x] 3.4.10 Шаг `just check-rust`
  - [x] 3.4.11 Шаг `just test-rust`
  - [x] 3.4.12 Шаг `just lint`
  - [x] 3.4.13 Шаг `just check-ui`
  - [x] 3.4.14 Шаг `just test-ui`
- [x] 3.5 Job `openspec-validate` на `ubuntu-latest`:
  - [x] 3.5.1 `actions/checkout@v4`
  - [x] 3.5.2 Установить openspec CLI: `npm install -g @fission-ai/openspec` (пакет, который ставится глобально через homebrew/npm в локали)
  - [x] 3.5.3 Перебрать все директории `openspec/changes/*` (кроме `archive/`) и запустить `openspec validate <change> --strict`; CI падает, если хотя бы одна не прошла
- [x] 3.6 Job `security-audit` на `ubuntu-latest`:
  - [x] 3.6.1 `taiki-e/install-action@v2` для `cargo-audit`
  - [x] 3.6.2 Шаг `cargo audit --file src-tauri/Cargo.lock` (без `--deny warnings` — см. design D6 и комментарий в `.cargo/audit.toml`; vulnerabilities падают gate, warnings — informational)
  - [x] 3.6.3 `actions/setup-node@v4` + `npm ci`
  - [x] 3.6.4 Шаг `npm audit --omit=dev --audit-level=high`
- [x] 3.7 Job `tauri-build-smoke` с матрицей `os: [ubuntu-latest, macos-latest, windows-latest]`, `fail-fast: false`:
  - [x] 3.7.1 Установка системных зависимостей (Linux-only step через `if: matrix.os == 'ubuntu-latest'`)
  - [x] 3.7.2 `dtolnay/rust-toolchain@master`, `Swatinem/rust-cache@v2`, `setup-node@v4`, `npm ci`
  - [x] 3.7.3 Шаг `npm run tauri build -- --no-bundle`
  - [x] 3.7.4 `defaults.run.shell: bash` на уровне workflow, чтобы pwsh-цитирование не ломало команды на Windows
- [x] 3.8 Job-ы независимые (`needs:` нет ни у одного), запускаются параллельно

## 4. Release workflow (`.github/workflows/release.yml`)

- [x] 4.1 Триггер: `push.tags: ['v*']`
- [x] 4.2 `permissions: { contents: write }`
- [x] 4.3 Concurrency без cancel-in-progress: `group: release-${{ github.workflow }}-${{ github.ref }}`, `cancel-in-progress: false`
- [x] 4.4 Job `release` с матрицей:
  - `{ platform: macos-latest, args: '--target universal-apple-darwin' }`
  - `{ platform: windows-latest, args: '' }`
- [x] 4.5 Шаги job:
  - [x] 4.5.1 `actions/checkout@v4`
  - [x] 4.5.2 `dtolnay/rust-toolchain@master` (`targets: aarch64-apple-darwin,x86_64-apple-darwin` только для macOS, через conditional expression)
  - [x] 4.5.3 `Swatinem/rust-cache@v2` с `key: release-${{ matrix.platform }}`
  - [x] 4.5.4 `actions/setup-node@v4` (`.nvmrc`), `npm ci`
  - [x] 4.5.5 `tauri-apps/tauri-action@v0` с `tagName: ${{ github.ref_name }}`, `releaseName: ${{ github.ref_name }}`, `releaseDraft: true`, `args: ${{ matrix.args }}`
  - [x] 4.5.6 `env: { GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }} }`

## 5. Документация

- [x] 5.1 Создать `docs/ci.md`:
  - [x] 5.1.1 Список всех CI-гейтов (Rust / frontend / OpenSpec / audit / smoke build)
  - [x] 5.1.2 Триггер-матрица (PR / push / dispatch / tag)
  - [x] 5.1.3 Релизный процесс: как создать тег `v*`, что попадёт в draft release, как опубликовать
  - [x] 5.1.4 Процедура апдейта `rust-toolchain.toml` и `.nvmrc` (что проверить локально)
  - [x] 5.1.5 Как добавить ignore в `.cargo/audit.toml` (формат + требование justification)
  - [x] 5.1.6 Ожидаемые тайминги по job-ам (таблица «typical / worst case»)
- [x] 5.2 Обновить `AGENTS.md` в разделе "Commands" / "Documentation":
  - [x] 5.2.1 Добавить ссылку на `docs/ci.md`
  - [x] 5.2.2 Упомянуть, что CI зовёт те же `just` recipes (локально и на runner идентично) + добавить `just check-rust`, `just check-ui`, `just lint`, `just ci` в таблицу команд
- [x] 5.3 Обновить `CLAUDE.md` в разделе "Workflow": добавить пункт «перед PR прогнать `just ci` (или индивидуально `just clippy`/`check`/`lint`/`test`) и убедиться, что `openspec validate --strict` проходит для активных изменений — иначе CI заблокирует мерж»
- [x] 5.4 Добавить в `justfile` рецепты `check-rust`, `check-ui`, `check` (composite), `lint`, `fmt`, `ci: clippy check lint test`

## 6. Верификация (требует push в GitHub — выполнять на этапе review PR)

- [ ] 6.1 Запушить change в feature-ветку (`feature/add-github-ci`), открыть PR в `main` — убедиться, что все job-ы зелёные на cached-run < 15 мин
- [ ] 6.2 Проверить `workflow_dispatch` вручную из GitHub UI на feature-ветке
- [ ] 6.3 На временной ветке создать тестовый тег `v0.0.0-ci-test`, проверить, что `release.yml` собирает артефакты под macOS+Windows и создаёт draft release. После проверки — удалить тестовый release и тег (`git tag -d v0.0.0-ci-test; git push origin :refs/tags/v0.0.0-ci-test`)
- [ ] 6.4 Намеренно сломать clippy (например, добавить `let _x: i32 = "hello";` в тестовом коммите) — убедиться, что CI падает на этом гейте. Откатить коммит
- [ ] 6.5 Намеренно сломать OpenSpec (добавить change без `tasks.md`) — убедиться, что `openspec-validate` job падает. Откатить
- [x] 6.6 Записать в `docs/ci.md` инструкцию, как настроить branch protection rules в GitHub UI (required checks: lint-and-test, openspec-validate, security-audit, tauri-build-smoke / все три ОС) — см. секцию "Branch protection rules (manual setup)" в `docs/ci.md`

## 7. Финализация

- [x] 7.1 Прогнать `openspec validate add-github-ci --strict` локально
- [x] 7.2 Проверить, что все файлы созданы и согласованы (`proposal.md`, `design.md`, `tasks.md`, `specs/ci-pipeline/spec.md`)
- [ ] 7.3 После мержа PR — выполнить `openspec archive add-github-ci`
