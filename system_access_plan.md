# System Access Plan: Full Control for Code Execution

## 1. Introduction

This document outlines a plan to implement "Full Control / Unlimited Access for Code Execution System Wide" for the Master Orchestrator (MO). This capability will grant the MO the ability to execute any command, anywhere on the user's system, providing a powerful tool for development, administration, and automation.

Given the significant security implications of such a feature, this plan emphasizes a multi-layered approach to security, ensuring that the user has explicit control over when and how these elevated privileges are activated.

## 2. Guiding Principles

*   **Explicit User Consent:** The user must explicitly and unambiguously grant elevated privileges. There will be no "on by default" behavior.
*   **Clear Visual Indicators:** The user must always be aware of when the system is operating in a high-privilege mode.
*   **Layered Security:** Multiple, independent security measures will be implemented to prevent accidental or malicious use of elevated privileges.
*   **Auditability:** All actions taken with elevated privileges must be logged and auditable.

## 3. Technical Implementation

### 3.1. Tiered Access Levels

To provide granular control, we will introduce a tiered access system.

*   **Tier 0: Standard Access (Default)**
    *   The MO operates with the standard, restricted access levels.
    *   No access to the file system outside of the workspace.
    *   `execute_command` is sandboxed and cannot affect the system.

*   **Tier 1: File System Access**
    *   Grants full read/write access to the local file system.
    *   Activated by the existing `MASTER_ORCHESTRATOR_FULL_ACCESS` environment variable.
    *   This is the current "Full System Access" functionality.

*   **Tier 2: Unrestricted Code Execution**
    *   Grants the ability to execute any command, anywhere on the system.
    *   This is the new "Full Control" mode.
    *   Requires a separate, more explicit activation mechanism.

### 3.2. Activation and Control

Activating Tier 2 will require two distinct user actions:

1.  **Environment Variable:** The user must set a new environment variable, `MASTER_ORCHESTRATOR_UNRESTRICTED_EXECUTION`, to `true`. This is a conscious, out-of-band action that signals intent.

2.  **Live User Confirmation:** When a request is made to execute a command in Tier 2, the user will be presented with a clear, unambiguous confirmation dialog. This dialog will:
    *   Display the exact command to be executed.
    *   Warn the user of the potential consequences.
    *   Require an explicit "Allow" or "Deny" response for each command.
    *   Include a "Remember my choice for this session" option to reduce friction for trusted workflows.

### 3.3. New `execute_unrestricted_command` Tool

A new tool, `execute_unrestricted_command`, will be introduced. This tool will be the sole entry point for executing commands in Tier 2.

```python
def execute_unrestricted_command(
    command: str,
    working_directory: str = None,
) -> dict:
    """
    Executes a command with unrestricted system-wide privileges.
    WARNING: This tool can execute any command, anywhere on the system.
    Use with extreme caution.
    """
```

This tool will only be available when Tier 2 is active. Any attempt to use it outside of Tier 2 will result in an error.

### 3.4. Security and Safety

*   **Visual Indicators:** When Tier 2 is active, the user interface will display a prominent, persistent visual indicator (e.g., a red border around the screen, a blinking icon in the status bar) to remind the user that the system is in a high-privilege state.
*   **Logging and Auditing:** Every call to `execute_unrestricted_command` will be logged in a dedicated, tamper-evident audit trail. The log entry will include:
    *   The full command that was executed.
    *   The user who authorized the command.
    *   The timestamp of the execution.
    *   The result of the command.
*   **Timeout/Inactivity Lock:** If Tier 2 is active but there is no user interaction for a configurable period (e.g., 15 minutes), the system will automatically revert to Tier 0 and require re-authentication to re-activate Tier 2.

## 4. User Experience (UX) Flow

1.  **User wishes to enable unrestricted execution.**
2.  **User sets `MASTER_ORCHESTRATOR_UNRESTRICTED_EXECUTION=true` in their environment.**
3.  **The MO proposes a command using `execute_unrestricted_command`.**
4.  **The frontend displays a prominent confirmation dialog:**
    *   "The Master Orchestrator is requesting to execute the following command with unrestricted system privileges:"
    *   `[command to be executed]`
    *   "WARNING: This command will be executed with full access to your system and could have unintended consequences. Are you sure you want to proceed?"
    *   Buttons: "Allow Once", "Allow for this Session", "Deny"
5.  **User selects an option.**
6.  **If allowed, the command is executed and the result is returned.**
7.  **The entire transaction is logged.**
8.  **The visual indicator of Tier 2 access remains active.**

## 5. Next Steps

1.  **Implement the `execute_unrestricted_command` tool.**
2.  **Develop the frontend confirmation dialog and visual indicators.**
3.  **Implement the tiered access logic and the environment variable checks.**
4.  **Create the secure logging and auditing mechanism.**
5.  **Thoroughly test the entire system for security vulnerabilities and usability issues.**
