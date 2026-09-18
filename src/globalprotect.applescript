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
                set targetStatus to my globalProtectString("Connected")
                set buttonName to my globalProtectString("Connect")
            else if operation is "disconnect" then
                set targetStatus to my globalProtectString("Not Connected")
                set buttonName to my globalProtectString("Disconnect")
            else
                error "Unsupported operation"
            end if
            if currentStatus is targetStatus then return my finishOperation(operation, currentStatus)

            -- If a transition is already in progress, wait rather than click again.
            set transitionInProgress to false
            if operation is "connect" and (currentStatus is my globalProtectString("Connecting") or currentStatus is my globalProtectString("Connecting...")) then set transitionInProgress to true
            if operation is "disconnect" and currentStatus is my globalProtectString("Disconnecting...") then set transitionInProgress to true
            if not transitionInProgress then
                if not (exists first window) then click menu bar item 1 of menu bar 2
                repeat
                    if help of menu bar item 1 of menu bar 2 is targetStatus then return my finishOperation(operation, targetStatus)
                    if exists first window then
                        if exists button buttonName of first window then
                            if enabled of button buttonName of first window then exit repeat
                        end if
                    end if
                    delay 0.2
                end repeat
                click button buttonName of first window
            end if

            -- Rust enforces an overall deadline, including UI and authentication waits.
            repeat until help of menu bar item 1 of menu bar 2 is targetStatus
                delay 0.5
            end repeat
            return my finishOperation(operation, targetStatus)
        end tell
    end tell
end performOperation

-- Resolve labels from GlobalProtect's Localizable.strings for the user's
-- preferred language rather than depending on English AX names.
on globalProtectString(key)
    return localized string key from table "Localizable" in bundle (path to application "GlobalProtect")
end globalProtectString

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
