This is a llama.cpp launcher app

- the values that are Option<T> are to be represented as a pair or a checkbox (whether to define them) and a control that is active when the checkbox is checked
- if the checkbox is false, None is set
- if it is checked, Some(...) is set, default initially
- Message for value is Message::ValueNameV, while for the checkbox it is Message::ValueName
- if a value is None, it should not be set for the llama-server

- model file value is to be set by using the button, that will run rfd file dialogue (preferably asynchronous) to look for the file needed (.gguf files)
- same for the mmproj, it is to have a similar button to look for files

- Host value is to be validated, if not a valid ipv4 or ipv6 address, it should not be passed

## llama-server process management
- there should be a sipper and subscription that will poll every 2 seconds for a running process and depending on the status, return the value
- values:

⬜️ - local llama.cpp found, not running
🟩 - running, ready
🟥 - starting, not ready, not responding
🟦 - running, ready, but not started from this app
⬛️ - not available

The button to start should deactivate if own llama-server process has been running.
If another llama-server is running but has not been started by this app (App::child field is None) then it should be inactive

## languages
- the app uses rust_i18n
- thre should be a language pick_list to select desired UI language
- languages to be supported (locales directory): EN_US, DE, FR, ES, IT, PL, RU, ZH_CN, JP, TR, AR, KO, PT
