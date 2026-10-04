Feature: Notifications on mobile
  Source: ADR 0004, roadmap M5-03; the end of a session is announced by a scheduled notification

  @ui @REQ-13 @mobile
  Scenario: The first timer start asks for the notification permission
    Given the platform is "mobile"
    And the notification permission is "prompt"
    And the user answers the permission dialog with "granted"
    When I open the app
    And I click "Start"
    Then the backend received "request_notification_permission"
    And the backend received "start_timer"
    And I do not see "Benachrichtigungen sind blockiert"

  @ui @REQ-13 @mobile
  Scenario: A denied permission does not stop the timer but is explained once
    Given the platform is "mobile"
    And the notification permission is "prompt"
    And the user answers the permission dialog with "denied"
    When I open the app
    And I click "Start"
    Then the backend received "start_timer"
    And I see "Benachrichtigungen sind blockiert"

  @ui @REQ-13 @mobile
  Scenario: A granted permission is not requested again
    Given the platform is "mobile"
    And the notification permission is "granted"
    When I open the app
    And I click "Start"
    Then the backend received "start_timer"
    And the backend did not receive "request_notification_permission"

  @ui @REQ-13 @mobile
  Scenario: Disabled notifications never ask for the permission
    Given the platform is "mobile"
    And the notification permission is "prompt"
    And the setting "notificationsEnabled" is "false"
    When I open the app
    And I click "Start"
    Then the backend received "start_timer"
    And the backend did not receive "request_notification_permission"

  @ui @REQ-13 @desktop
  Scenario: The desktop never asks for the notification permission
    Given the platform is "desktop"
    When I open the app
    And I click "Start"
    Then the backend received "start_timer"
    And the backend did not receive "get_notification_status"
    And the backend did not receive "request_notification_permission"

  @ui @REQ-13 @mobile
  Scenario: The settings offer to grant a pending permission
    Given the platform is "mobile"
    And the notification permission is "prompt"
    And the user answers the permission dialog with "granted"
    When I open the app at "/settings"
    Then I see the buttons "Benachrichtigungen erlauben"
    When I click "Benachrichtigungen erlauben"
    Then the backend received "request_notification_permission"
    And I do not see the button "Benachrichtigungen erlauben"

  @ui @REQ-13 @mobile
  Scenario: A blocked permission is explained on the timer view
    Given the platform is "mobile"
    And the notification permission is "denied"
    And exact alarms are allowed
    When I open the app
    Then I see "Benachrichtigungen sind für nunc ausgeschaltet"
    And I see the buttons "Systemeinstellungen öffnen"
    When I click "Systemeinstellungen öffnen"
    Then Android opens the notification settings

  @ui @REQ-13 @mobile
  Scenario: Missing exact alarms are explained and can be enabled
    Given the platform is "mobile"
    And exact alarms are not allowed
    When I open the app
    Then I see "Genaue Alarme sind ausgeschaltet"
    When I click "Alarme erlauben"
    Then Android opens the exact alarm settings

  @ui @REQ-13 @mobile
  Scenario: Nothing is shown when everything is allowed
    Given the platform is "mobile"
    When I open the app
    Then I do not see "ausgeschaltet"
    And I do not see "blockiert"
