Feature: Flowtime break recommendation
  Source: STORIES US-FT-02, REQUIREMENTS 2.2; nunc-go timer_pure_test.go

  @domain @US-FT-02 @wip
  Scenario: Break is the floor of worked minutes times the ratio

  @domain @US-FT-02 @wip
  Scenario: No break is suggested below the minimum flow state

  @domain @US-FT-02 @wip
  Scenario: A session of exactly the minimum flow state suggests a break

  @domain @US-FT-02 @wip
  Scenario: Ratio limits 0.1 and 0.5 are accepted and values outside are rejected

  @domain @US-FT-02 @wip
  Scenario: Flowtime counts up instead of down

  @domain @US-FT-02 @wip
  Scenario: Stopping a flowtime session completes it and suggests a break

  @domain @US-FT-02 @wip
  Scenario: The suggested break can be started as a countdown

  @integration @US-FT-02 @wip
  Scenario: Default flowtime settings are ratio 0.2 and minimum flow state 15

  @ui @US-FT-02 @wip
  Scenario: The ratio slider ranges from 1:0.1 to 1:0.5 in steps of 0.05

  @ui @US-FT-02 @wip
  Scenario: Anchor labels STRICT, BALANCED and RELAXED are shown below the slider

  @ui @US-FT-02 @wip
  Scenario: The selected ratio is shown as a badge

  @ui @US-FT-02 @wip
  Scenario: The minimum flow state accepts 1 to 60 minutes

  @ui @US-FT-02 @wip
  Scenario: The dynamic break scaling info banner is shown

  @ui @US-FT-02 @wip
  Scenario: The stopwatch is displayed while a flowtime session runs

  @ui @US-FT-02 @wip
  Scenario: The break recommendation is shown after stopping
