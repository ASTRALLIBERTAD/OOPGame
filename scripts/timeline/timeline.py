import json
import pandas as pd
import plotly.graph_objects as go
from datetime import datetime
import textwrap

TARGET_FINISH_DATE = "2026-06-30"
VS_BG = "#1E1E1E"        # Editor Background
VS_HEADER = "#252526"    # Header Background
VS_BORDER = "#454545"    # Borders
VS_TEXT = "#CCCCCC"      # General Text
VS_BLUE = "#4FC1FF"      # Variable Blue

with open("../../docs/issues.json") as f:
    issues = json.load(f)

processed_tasks = []
start_dates = []

for issue in issues:
    title = issue["title"]
    created_at = issue["createdAt"][:10]
    start_dates.append(created_at)
    
    assignees = [a['login'] for a in issue.get('assignees', [])]
    assigned_to = ", ".join(assignees) if assignees else "Unassigned"
    
    # Status Logic and Colors
    is_done = issue["state"] == "CLOSED"
    status_label = "DONE" if is_done else "ACTIVE"
    # Red (#F44747) for Active, Green (#608B4E) for Done
    status_color = "#608B4E" if is_done else "#F44747"

    processed_tasks.append({
        "Task": textwrap.fill(title, 55).replace('\n', '<br>'),
        "Status": status_label,
        "S_Color": status_color,
        "Assigned": assigned_to,
        "Date": created_at,
        "IsDone": is_done
    })

df = pd.DataFrame(processed_tasks)
df = df.sort_values(by=["IsDone", "Date"], ascending=[True, False])

project_start = min(start_dates) if start_dates else "N/A"

fig = go.Figure(data=[go.Table(
    columnwidth=[450, 100, 150, 120],
    header=dict(
        values=["<b>FILE / TASK</b>", "<b>STATUS</b>", "<b>ASSIGNED</b>", "<b>STARTED</b>"],
        fill_color=VS_HEADER,
        align='left',
        font=dict(color=VS_TEXT, size=13, family="Consolas, monospace"),
        line_color=VS_BORDER
    ),
    cells=dict(
        values=[df.Task, df.Status, df.Assigned, df.Date],
        fill_color=[
            [VS_BG] * len(df), 
            df.S_Color, 
            [VS_BG] * len(df), 
            [VS_BG] * len(df)
        ],
        align='left',
        font=dict(
            # Dark text for Status cells, Blue for Tasks, Light Grey for others
            color=[[VS_BLUE] * len(df), ["#1E1E1E"] * len(df), [VS_TEXT] * len(df), [VS_TEXT] * len(df)],
            size=12, 
            family="Consolas, monospace"
        ),
        line_color=VS_BORDER,
        height=40
    ))
])

fig.update_layout(
    title={
        'text': f"<b style='color:{VS_BLUE};'>TIMELINE</b><br><span style='color:#858585; font-size:14px;'>Start: {project_start} | Target: {TARGET_FINISH_DATE}</span>",
        'x': 0.02, 'y': 0.95
    },
    margin=dict(l=15, r=15, t=100, b=15),
    width=1000,
    height=max(400, len(df) * 50 + 150),
    paper_bgcolor=VS_BG,
)

fig.write_image("../../docs/timeline.png", scale=2)
