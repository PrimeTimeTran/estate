```text
                         View
                          │
                     ViewType
                          │
              ┌───────────┴───────────┐
              │                       │
       ProblemScreen            DashboardScreen
              │                       │
         implements               implements
           Screen                    Screen
              │                       │
              ▼                       ▼
             Ve                      Ve
              │
       owns 8 regions
              │
       ┌──────┼────────┬──────────┐
       ▼      ▼        ▼          ▼
     Panel   Panel    Panel      Panel
       │      │        │          │
       ▼      ▼        ▼          ▼
   Veable   Veable   Veable     Veable
       │      │        │
       ▼      ▼        ▼
    Sidebar Editor   Output
```
