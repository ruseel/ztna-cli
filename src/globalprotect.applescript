-- Labels copied from the installed GlobalProtect 6.3.3 Localizable.strings
-- bundles. Keeping them in this embedded script avoids any runtime bundle lookup.
property connectLabels : {"Connect", "Verbinden", "连接", "接続", "Conectar", "연결", "連線", "Connecter"}
property disconnectLabels : {"Disconnect", "Trennen", "中断连接", "切断", "Desconectar", "연결 해제", "中斷連線", "Déconnecter"}
property connectedLabels : {"Connected", "Verbunden", "已连接", "接続済み", "Conectado", "연결됨", "已連線", "Connecté"}
property disconnectedLabels : {"Not Connected", "Nicht verbunden", "没有连接", "未接続", "No conectado", "연결되지 않음", "沒有連線", "Non connecté"}
property connectingLabels : {"Connecting", "Connecting...", "Verbinden", "Verbinde...", "连接中", "正在连接...", "接続中", "接続中...", "Conectando", "Conectando...", "연결 중", "연결 중...", "連線中", "正在連線...", "Connexion", "Connexion en cours..."}
property disconnectingLabels : {"Disconnecting...", "Verbindung wird getrennt", "正在断开连接...", "切断中...", "Desconectando...", "연결 해제 중...", "正在中斷連線...", "Déconnexion..."}

on enabledButtonName(panel, labelList)
    tell application "System Events"
        repeat with labelText in labelList
            set buttonName to contents of labelText
            if exists button buttonName of panel then
                if enabled of button buttonName of panel then return buttonName
            end if
        end repeat
    end tell
    return missing value
end enabledButtonName

on performOperation(operation)
    tell application "System Events"
        if not (exists process "GlobalProtect") then
            if operation is "status" then return "Not running"
            error "GlobalProtect is not running. Open the GlobalProtect app first."
        end if
        tell process "GlobalProtect"
            -- Same status source as ~/bin/globalprotect-{connect,disconnect}.applescript.
            set currentStatus to help of menu bar item 1 of menu bar 2
            if operation is "status" then return currentStatus

            if operation is "connect" then
                set targetStatuses to connectedLabels
                set buttonNames to connectLabels
                set transitionStatuses to connectingLabels
            else if operation is "disconnect" then
                set targetStatuses to disconnectedLabels
                set buttonNames to disconnectLabels
                set transitionStatuses to disconnectingLabels
            else
                error "Unsupported operation"
            end if
            if currentStatus is in targetStatuses then return my finishOperation(operation, currentStatus)

            -- If a transition is already in progress, wait rather than click again.
            set transitionInProgress to (currentStatus is in transitionStatuses)
            if not transitionInProgress then
                if not (exists first window) then click menu bar item 1 of menu bar 2
                repeat
                    set currentStatus to help of menu bar item 1 of menu bar 2
                    if currentStatus is in targetStatuses then return my finishOperation(operation, currentStatus)
                    if exists first window then
                        set buttonName to my enabledButtonName(first window, buttonNames)
                        if buttonName is not missing value then exit repeat
                    end if
                    delay 0.2
                end repeat
                click button buttonName of first window
            end if

            -- Rust enforces an overall deadline, including UI and authentication waits.
            repeat
                set currentStatus to help of menu bar item 1 of menu bar 2
                if currentStatus is in targetStatuses then exit repeat
                delay 0.5
            end repeat
            return my finishOperation(operation, currentStatus)
        end tell
    end tell
end performOperation

on finishOperation(operation, currentStatus)
    if operation is "disconnect" then
        tell application "System Events"
            tell process "GlobalProtect"
                if exists first window then
                    -- Dismiss only GlobalProtect's menu-bar panel. Never send a
                    -- global Cmd+W, which could close a different app's window.
                    click menu bar item 1 of menu bar 2
                end if
            end tell
        end tell
    end if
    return currentStatus
end finishOperation
