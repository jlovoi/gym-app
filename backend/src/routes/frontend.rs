use axum::extract::State;
use axum::response::Html;
use axum::routing::get;
use axum::Router;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(index))
}

async fn index(State(state): State<AppState>) -> Html<String> {
    Html(page(&state.config.clerk_publishable_key))
}

fn page(clerk_key: &str) -> String {
    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Gym</title>
<style>
* {{ margin: 0; padding: 0; box-sizing: border-box; }}
body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; background: #0a0a0a; color: #e5e5e5; min-height: 100vh; }}
.container {{ max-width: 720px; margin: 0 auto; padding: 24px 16px; }}
h1 {{ font-size: 1.5rem; margin-bottom: 8px; }}
h2 {{ font-size: 1.1rem; color: #a3a3a3; margin-bottom: 16px; font-weight: 400; }}
.card {{ background: #171717; border: 1px solid #262626; border-radius: 12px; padding: 20px; margin-bottom: 16px; }}
.card h3 {{ font-size: 0.85rem; text-transform: uppercase; letter-spacing: 0.05em; color: #737373; margin-bottom: 12px; }}
.badge {{ display: inline-block; padding: 2px 10px; border-radius: 999px; font-size: 0.75rem; font-weight: 600; }}
.badge.active {{ background: #065f46; color: #6ee7b7; }}
.badge.inactive {{ background: #7f1d1d; color: #fca5a5; }}
.badge.role {{ background: #1e3a5f; color: #93c5fd; }}
button {{ background: #fff; color: #0a0a0a; border: none; padding: 10px 20px; border-radius: 8px; font-size: 0.9rem; font-weight: 600; cursor: pointer; transition: opacity 0.15s; }}
button:hover {{ opacity: 0.85; }}
button:disabled {{ opacity: 0.4; cursor: not-allowed; }}
button.secondary {{ background: #262626; color: #e5e5e5; }}
button.danger {{ background: #7f1d1d; color: #fca5a5; }}
button.small {{ padding: 6px 14px; font-size: 0.8rem; }}
.workout-desc {{ font-size: 1.1rem; line-height: 1.6; margin: 8px 0; white-space: pre-line; }}
.class-row {{ display: flex; justify-content: space-between; align-items: center; padding: 10px 0; border-bottom: 1px solid #262626; }}
.class-row:last-child {{ border-bottom: none; }}
.class-time {{ font-weight: 600; }}
.class-spots {{ color: #737373; font-size: 0.85rem; }}
.log-form {{ display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }}
.log-form input {{ background: #262626; border: 1px solid #404040; color: #e5e5e5; padding: 8px 12px; border-radius: 8px; font-size: 0.9rem; width: 120px; }}
.log-form label {{ font-size: 0.85rem; color: #a3a3a3; display: flex; align-items: center; gap: 4px; }}
.leaderboard-row {{ display: flex; justify-content: space-between; padding: 6px 0; border-bottom: 1px solid #1a1a1a; }}
.leaderboard-row:last-child {{ border-bottom: none; }}
.rank {{ color: #737373; width: 24px; }}
.msg {{ padding: 12px 16px; border-radius: 8px; margin-bottom: 12px; font-size: 0.85rem; }}
.msg.error {{ background: #7f1d1d; color: #fca5a5; }}
.msg.success {{ background: #065f46; color: #6ee7b7; }}
.msg.info {{ background: #1e3a5f; color: #93c5fd; }}
#auth-area {{ text-align: center; padding: 80px 0; }}
#auth-area h1 {{ font-size: 2rem; margin-bottom: 4px; }}
#auth-area p {{ color: #737373; margin-bottom: 32px; }}
.hidden {{ display: none; }}
.admin-form {{ display: flex; flex-direction: column; gap: 10px; }}
.admin-form input, .admin-form textarea, .admin-form select {{ background: #262626; border: 1px solid #404040; color: #e5e5e5; padding: 8px 12px; border-radius: 8px; font-size: 0.9rem; font-family: inherit; }}
.admin-form textarea {{ resize: vertical; }}
.custom-field-row {{ display: flex; gap: 8px; align-items: center; }}
.custom-field-row input, .custom-field-row select {{ flex: 1; background: #262626; border: 1px solid #404040; color: #e5e5e5; padding: 8px 12px; border-radius: 8px; font-size: 0.9rem; }}
#custom-fields-area {{ display: flex; flex-direction: column; gap: 8px; }}
.topbar {{ display: flex; justify-content: space-between; align-items: center; margin-bottom: 24px; }}
.topbar-user {{ display: flex; align-items: center; gap: 12px; }}
</style>
</head>
<body>

<div class="container">
  <!-- Signed-out state -->
  <div id="auth-area">
    <h1>Gym</h1>
    <p>Sign in to view workouts, classes, and log your scores.</p>
    <div id="sign-in-btn"></div>
  </div>

  <!-- Signed-in state -->
  <div id="app" class="hidden">
    <div class="topbar">
      <h1>Gym</h1>
      <div class="topbar-user">
        <span id="user-role" class="badge role"></span>
        <span id="user-status" class="badge"></span>
        <button class="secondary small" id="sign-out-btn">Sign Out</button>
      </div>
    </div>

    <div id="msg-area"></div>

    <!-- Profile -->
    <div class="card">
      <h3>Profile</h3>
      <div id="profile-info"></div>
    </div>

    <!-- Today's workout -->
    <div class="card">
      <h3>Today's Workout</h3>
      <div id="workout-area"></div>
    </div>

    <!-- Log result -->
    <div class="card" id="log-card" style="display:none">
      <h3>Log Your Result</h3>
      <div id="log-form-area"></div>
    </div>

    <!-- Leaderboard -->
    <div class="card" id="leaderboard-card" style="display:none">
      <h3>Leaderboard</h3>
      <div id="leaderboard-area"></div>
    </div>

    <!-- Classes -->
    <div class="card">
      <h3>Today's Classes</h3>
      <div id="classes-area"></div>
    </div>

    <!-- Membership -->
    <div class="card">
      <h3>Membership</h3>
      <div id="membership-area"></div>
    </div>

    <!-- Admin: Create Workout (staff+) -->
    <div class="card hidden" id="admin-workout-card">
      <h3>Create Workout</h3>
      <div class="admin-form">
        <input type="date" id="workout-date">
        <textarea id="workout-desc" placeholder="Workout description (e.g. 21-15-9 Thrusters &amp; Pull-ups)" rows="3"></textarea>
        <select id="workout-score-type" onchange="onScoreTypeChange()">
          <option value="time">Time (MM:SS)</option>
          <option value="reps">Reps</option>
          <option value="weight">Weight (lbs)</option>
          <option value="rounds_reps">Rounds + Reps</option>
          <option value="distance">Distance (m)</option>
          <option value="custom">Custom</option>
        </select>
        <input type="text" id="workout-score-label" placeholder="Score label (optional, e.g. Total lbs)">
        <div id="custom-fields-builder" style="display:none">
          <div id="custom-fields-area"></div>
          <button type="button" class="small secondary" onclick="addCustomField()">+ Add Field</button>
        </div>
        <button class="small" onclick="createWorkout()">Post Workout</button>
      </div>
    </div>

    <!-- Admin: Schedule Class (admin) -->
    <div class="card hidden" id="admin-class-card">
      <h3>Schedule Class</h3>
      <div class="admin-form">
        <input type="date" id="class-date">
        <div style="display:flex;gap:8px">
          <input type="time" id="class-start" value="09:00">
          <input type="time" id="class-end" value="10:00">
        </div>
        <input type="number" id="class-capacity" placeholder="Capacity" value="20" min="1">
        <button class="small" onclick="createClass()">Schedule Class</button>
      </div>
    </div>
  </div>
</div>

<script
  async
  crossorigin="anonymous"
  data-clerk-publishable-key="{clerk_key}"
  src="https://cdn.jsdelivr.net/npm/@clerk/clerk-js@latest/dist/clerk.browser.js"
  onload="clerkLoaded()"
></script>

<script>
const API = '';
let currentWorkoutId = null;
let currentUserRole = null;
let currentScoreType = 'time';
let currentScoreConfig = null;
let currentScoreLabel = null;

function clerkLoaded() {{
  window.Clerk.load().then(() => {{
    if (window.Clerk.user) {{
      showApp();
    }} else {{
      window.Clerk.mountSignIn(document.getElementById('sign-in-btn'), {{
        afterSignInUrl: '/',
        afterSignUpUrl: '/',
      }});
    }}
  }});
}}

document.getElementById('sign-out-btn')?.addEventListener('click', () => {{
  window.Clerk.signOut().then(() => location.reload());
}});

async function api(path, opts = {{}}) {{
  const token = await window.Clerk.session.getToken();
  const headers = {{ 'Authorization': 'Bearer ' + token }};
  if (opts.body) headers['Content-Type'] = 'application/json';
  const res = await fetch(API + path, {{
    method: opts.method || 'GET',
    headers,
    body: opts.body ? JSON.stringify(opts.body) : undefined,
  }});
  const data = await res.json().catch(() => null);
  return {{ ok: res.ok, status: res.status, data }};
}}

function showMsg(text, type) {{
  const el = document.getElementById('msg-area');
  el.innerHTML = `<div class="msg ${{type}}">${{text}}</div>`;
  setTimeout(() => el.innerHTML = '', 5000);
}}

async function showApp() {{
  document.getElementById('auth-area').classList.add('hidden');
  document.getElementById('app').classList.remove('hidden');

  const params = new URLSearchParams(window.location.search);
  if (params.get('checkout') === 'success') {{
    showMsg('Payment successful! Your membership is now active.', 'success');
    window.history.replaceState(null, '', '/');
  }} else if (params.get('checkout') === 'cancel') {{
    showMsg('Checkout cancelled.', 'info');
    window.history.replaceState(null, '', '/');
  }}

  await Promise.all([loadProfile(), loadWorkout(), loadClasses()]);
}}

async function loadProfile() {{
  const {{ ok, data }} = await api('/users/me');
  if (!ok) {{
    document.getElementById('profile-info').textContent = 'Could not load profile. You may need to sign up first.';
    return;
  }}
  const user = data.user;
  const membership = data.membership;

  document.getElementById('user-role').textContent = user.role;
  const statusEl = document.getElementById('user-status');
  statusEl.textContent = user.is_active ? 'Active' : 'Inactive';
  statusEl.className = 'badge ' + (user.is_active ? 'active' : 'inactive');

  currentUserRole = user.role;
  document.getElementById('profile-info').innerHTML =
    `<div>ID: <code>${{user.id}}</code></div>`;

  // Show admin panels based on role
  if (user.role === 'staff' || user.role === 'admin') {{
    document.getElementById('admin-workout-card').classList.remove('hidden');
  }}
  if (user.role === 'admin') {{
    document.getElementById('admin-class-card').classList.remove('hidden');
  }}

  // Set default dates to today
  const today = new Date().toISOString().split('T')[0];
  document.getElementById('workout-date').value = today;
  document.getElementById('class-date').value = today;

  // Membership
  const mArea = document.getElementById('membership-area');
  if (membership) {{
    mArea.innerHTML = `<span class="badge active">${{membership.plan_type}} &mdash; ${{membership.status}}</span>`;
  }} else {{
    mArea.innerHTML = `
      <p style="margin-bottom:12px;color:#a3a3a3">No active membership</p>
      <button class="small" onclick="checkout('unlimited')">Subscribe &mdash; Unlimited</button>
    `;
  }}
}}

async function loadWorkout() {{
  const area = document.getElementById('workout-area');
  const {{ ok, data, status }} = await api('/workouts/today');
  if (!ok) {{
    area.innerHTML = status === 404
      ? '<span style="color:#737373">No workout posted today</span>'
      : '<span style="color:#fca5a5">Error loading workout</span>';
    return;
  }}
  currentWorkoutId = data.id;
  currentScoreType = data.score_type || 'time';
  currentScoreConfig = data.score_config || null;
  currentScoreLabel = data.score_label || null;
  area.innerHTML = `<div class="workout-desc">${{data.description || 'Rest day'}}</div>`;
  document.getElementById('log-card').style.display = '';
  document.getElementById('leaderboard-card').style.display = '';
  renderLogForm();
  loadLeaderboard();
}}

function renderLogForm() {{
  const area = document.getElementById('log-form-area');
  let html = '<div class="log-form">';
  switch (currentScoreType) {{
    case 'time':
      html += `<input type="number" id="log-min" placeholder="Min" style="width:80px" min="0">
               <span style="color:#737373">:</span>
               <input type="number" id="log-sec" placeholder="Sec" style="width:80px" min="0" max="59">`;
      break;
    case 'rounds_reps':
      html += `<input type="number" id="log-rounds" placeholder="Rounds" style="width:100px" min="0">
               <span style="color:#737373">+</span>
               <input type="number" id="log-extra-reps" placeholder="Reps" style="width:100px" min="0">`;
      break;
    case 'custom':
      if (currentScoreConfig && Array.isArray(currentScoreConfig)) {{
        currentScoreConfig.forEach((field) => {{
          const ph = field.type === 'weight' ? 'lbs' : field.type === 'distance' ? 'meters' : field.type === 'time' ? 'seconds' : 'value';
          html += `<div style="display:flex;gap:6px;align-items:center;width:100%">
            <span style="color:#a3a3a3;font-size:0.85rem;min-width:80px">${{field.name}}</span>
            <input type="number" class="custom-log-field" data-field="${{field.name}}" placeholder="${{ph}}" style="width:100px">
          </div>`;
        }});
      }}
      break;
    case 'weight':
      html += '<input type="number" id="log-value" placeholder="lbs">';
      break;
    case 'reps':
      html += '<input type="number" id="log-value" placeholder="Reps">';
      break;
    case 'distance':
      html += '<input type="number" id="log-value" placeholder="Meters">';
      break;
  }}
  html += '<label><input type="checkbox" id="log-rx"> Rx</label>';
  html += '<button class="small" onclick="submitLog()">Submit</button>';
  html += '</div>';
  area.innerHTML = html;
}}

function formatScore(log) {{
  switch (currentScoreType) {{
    case 'time': {{
      if (log.primary_value == null) return '\u2014';
      const total = Math.round(log.primary_value);
      const m = Math.floor(total / 60);
      const s = total % 60;
      return `${{m}}:${{s.toString().padStart(2, '0')}}`;
    }}
    case 'reps':
      return `${{log.primary_value ?? '\u2014'}} reps`;
    case 'weight':
      return `${{log.primary_value ?? '\u2014'}} lbs`;
    case 'distance':
      return `${{log.primary_value ?? '\u2014'}} m`;
    case 'rounds_reps': {{
      if (!log.data) return '\u2014';
      return `${{log.data.rounds || 0}} + ${{log.data.extra_reps || 0}}`;
    }}
    case 'custom': {{
      const label = currentScoreLabel || 'Total';
      let text = `${{log.primary_value ?? 0}} ${{label}}`;
      if (log.data && typeof log.data === 'object') {{
        const parts = Object.entries(log.data).map(([k, v]) => `${{k}}: ${{v}}`).join(', ');
        text += ` <span style="color:#737373;font-size:0.8rem">(${{parts}})</span>`;
      }}
      return text;
    }}
    default:
      return `${{log.primary_value ?? '\u2014'}}`;
  }}
}}

async function loadLeaderboard() {{
  if (!currentWorkoutId) return;
  const {{ ok, data }} = await api(`/workouts/${{currentWorkoutId}}/logs`);
  const area = document.getElementById('leaderboard-area');
  if (!ok || !data.length) {{
    area.innerHTML = '<span style="color:#737373">No scores yet</span>';
    return;
  }}
  area.innerHTML = data.map((log, i) => {{
    const name = [log.first_name, log.last_name].filter(Boolean).join(' ') || 'Anonymous';
    return `<div class="leaderboard-row">
      <span><span class="rank">${{i+1}}.</span> ${{name}}</span>
      <span>${{formatScore(log)}}${{log.is_rx ? ' Rx' : ''}}</span>
    </div>`;
  }}).join('');
}}

async function submitLog() {{
  if (!currentWorkoutId) return;
  const isRx = document.getElementById('log-rx').checked;
  let body = {{ is_rx: isRx }};

  switch (currentScoreType) {{
    case 'time': {{
      const min = parseInt(document.getElementById('log-min').value) || 0;
      const sec = parseInt(document.getElementById('log-sec').value) || 0;
      if (min === 0 && sec === 0) {{ showMsg('Enter a time', 'error'); return; }}
      body.primary_value = min * 60 + sec;
      break;
    }}
    case 'rounds_reps': {{
      const rounds = parseInt(document.getElementById('log-rounds').value) || 0;
      const extraReps = parseInt(document.getElementById('log-extra-reps').value) || 0;
      body.data = {{ rounds, extra_reps: extraReps }};
      break;
    }}
    case 'custom': {{
      const fields = document.querySelectorAll('.custom-log-field');
      const data = {{}};
      fields.forEach(f => {{
        data[f.dataset.field] = parseFloat(f.value) || 0;
      }});
      body.data = data;
      break;
    }}
    default: {{
      const value = parseFloat(document.getElementById('log-value').value);
      if (isNaN(value)) {{ showMsg('Enter a score', 'error'); return; }}
      body.primary_value = value;
      break;
    }}
  }}

  const {{ ok, data, status }} = await api(`/workouts/${{currentWorkoutId}}/logs`, {{
    method: 'POST',
    body,
  }});
  if (ok) {{
    showMsg('Score logged!', 'success');
    renderLogForm();
    loadLeaderboard();
  }} else if (status === 402) {{
    showMsg('Active membership required to log scores', 'error');
  }} else if (status === 409) {{
    showMsg('You already logged this workout', 'info');
  }} else {{
    showMsg(data?.error || 'Error', 'error');
  }}
}}

async function loadClasses() {{
  const today = new Date().toISOString().split('T')[0];
  const {{ ok, data }} = await api(`/classes?date=${{today}}`);
  const area = document.getElementById('classes-area');
  if (!ok || !data.length) {{
    area.innerHTML = '<span style="color:#737373">No classes today</span>';
    return;
  }}
  area.innerHTML = data.map(c => {{
    const start = c.start_time.slice(0, 5);
    const end = c.end_time.slice(0, 5);
    const spots = c.capacity - (c.signup_count || 0);
    const badge = c.user_signed_up ? '<span class="badge active" style="margin-left:8px">Signed Up</span>' : '';
    const btn = c.user_signed_up
      ? `<button class="small danger" onclick="cancelSignup('${{c.id}}')">Cancel</button>`
      : `<button class="small" onclick="signup('${{c.id}}')">Sign Up</button>`;
    return `<div class="class-row">
      <div>
        <span class="class-time">${{start}} &ndash; ${{end}}</span>
        ${{badge}}
        <span class="class-spots">${{spots}} spot${{spots !== 1 ? 's' : ''}} left</span>
      </div>
      <div>${{btn}}</div>
    </div>`;
  }}).join('');
}}

async function signup(classId) {{
  const {{ ok, status, data }} = await api(`/classes/${{classId}}/signup`, {{ method: 'POST' }});
  if (ok) {{
    showMsg('Signed up!', 'success');
    loadClasses();
  }} else if (status === 402) {{
    showMsg('Active membership required to sign up for classes', 'error');
  }} else if (status === 409) {{
    showMsg(data?.error || 'Already signed up', 'info');
  }} else {{
    showMsg(data?.error || 'Error', 'error');
  }}
}}

async function cancelSignup(classId) {{
  const {{ ok, status, data }} = await api(`/classes/${{classId}}/signup`, {{ method: 'DELETE' }});
  if (ok) {{
    showMsg('Signup cancelled', 'success');
    loadClasses();
  }} else if (status === 404) {{
    showMsg('Not signed up for this class', 'info');
  }} else {{
    showMsg(data?.error || 'Error', 'error');
  }}
}}

async function createWorkout() {{
  const date = document.getElementById('workout-date').value;
  const description = document.getElementById('workout-desc').value.trim();
  const scoreType = document.getElementById('workout-score-type').value;
  const scoreLabel = document.getElementById('workout-score-label').value.trim();
  if (!date) {{ showMsg('Pick a date', 'error'); return; }}

  let body = {{
    date,
    description: description || null,
    score_type: scoreType,
    score_label: scoreLabel || null,
  }};

  if (scoreType === 'custom') {{
    const fields = [];
    document.querySelectorAll('.custom-field-row').forEach(row => {{
      const name = row.querySelector('.cf-name').value.trim();
      const type = row.querySelector('.cf-type').value;
      if (name) fields.push({{ name, type }});
    }});
    if (fields.length === 0) {{ showMsg('Add at least one custom field', 'error'); return; }}
    body.score_config = fields;
  }}

  const {{ ok, data }} = await api('/workouts', {{
    method: 'POST',
    body,
  }});
  if (ok) {{
    showMsg(`Workout posted for ${{date}}`, 'success');
    document.getElementById('workout-desc').value = '';
    loadWorkout();
  }} else {{
    showMsg(data?.error || 'Failed to create workout', 'error');
  }}
}}

async function createClass() {{
  const date = document.getElementById('class-date').value;
  const start = document.getElementById('class-start').value;
  const end = document.getElementById('class-end').value;
  const capacity = parseInt(document.getElementById('class-capacity').value) || 20;
  if (!date || !start || !end) {{ showMsg('Fill in date and times', 'error'); return; }}
  const {{ ok, data }} = await api('/classes', {{
    method: 'POST',
    body: {{ date, start_time: start + ':00', end_time: end + ':00', capacity }},
  }});
  if (ok) {{
    showMsg(`Class scheduled for ${{date}} ${{start}}&ndash;${{end}}`, 'success');
    loadClasses();
  }} else {{
    showMsg(data?.error || 'Failed to schedule class', 'error');
  }}
}}

function onScoreTypeChange() {{
  const type = document.getElementById('workout-score-type').value;
  document.getElementById('custom-fields-builder').style.display = type === 'custom' ? '' : 'none';
}}

function addCustomField() {{
  const area = document.getElementById('custom-fields-area');
  const row = document.createElement('div');
  row.className = 'custom-field-row';
  row.innerHTML = `
    <input type="text" class="cf-name" placeholder="Field name (e.g. Snatch)">
    <select class="cf-type">
      <option value="weight">Weight</option>
      <option value="reps">Reps</option>
      <option value="time">Time</option>
      <option value="distance">Distance</option>
    </select>
    <button class="small danger" onclick="this.parentElement.remove()" type="button">&times;</button>
  `;
  area.appendChild(row);
}}

async function checkout(plan) {{
  const {{ ok, data }} = await api('/memberships/checkout', {{
    method: 'POST',
    body: {{ plan_type: plan }},
  }});
  if (ok && data.url) {{
    window.location.replace(data.url);
  }} else {{
    showMsg(data?.error || 'Checkout failed', 'error');
  }}
}}
</script>
</body>
</html>"##,
        clerk_key = clerk_key
    )
}
