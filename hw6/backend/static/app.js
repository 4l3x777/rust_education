const API = '/api';
let currentRoomKey = null;

// ---- Rooms -----------------------------------------------------------------

async function fetchRooms() {
    const res = await fetch(`${API}/rooms`);
    const rooms = await res.json();
    const container = document.getElementById('rooms-list');
    container.innerHTML = '';

    if (rooms.length === 0) {
        container.innerHTML = '<p style="color:#888">Комнат нет. Добавьте первую комнату.</p>';
        return;
    }

    rooms.forEach(room => {
        const card = document.createElement('div');
        card.className = 'card';
        card.innerHTML = `
            <div class="card-title">${escapeHtml(room.name)}</div>
            <div class="card-subtitle">Устройств: ${room.device_count}</div>
            <div class="card-actions">
                <button onclick="openRoom('${escapeAttr(room.key)}')">Открыть</button>
                <button class="btn-danger" onclick="deleteRoom('${escapeAttr(room.key)}')">Удалить</button>
            </div>
        `;
        container.appendChild(card);
    });
}

function showRooms() {
    document.getElementById('rooms-view').classList.remove('hidden');
    document.getElementById('room-detail').classList.add('hidden');
    document.getElementById('report-view').classList.add('hidden');
    fetchRooms();
}

function showAddRoom() {
    document.getElementById('room-name').value = '';
    showModal('modal-add-room');
}

async function addRoom() {
    const name = document.getElementById('room-name').value.trim();
    if (!name) return;
    const res = await fetch(`${API}/rooms`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name }),
    });
    if (res.ok) {
        hideModal('modal-add-room');
        fetchRooms();
    } else {
        const err = await res.json();
        alert(err.error || 'Ошибка');
    }
}

async function deleteRoom(key) {
    if (!confirm(`Удалить комнату "${key}"?`)) return;
    const res = await fetch(`${API}/rooms/${encodeURIComponent(key)}`, { method: 'DELETE' });
    if (res.ok) fetchRooms();
    else alert('Ошибка удаления');
}

// ---- Room detail -----------------------------------------------------------

async function openRoom(key) {
    currentRoomKey = key;
    const res = await fetch(`${API}/rooms/${encodeURIComponent(key)}`);
    if (!res.ok) {
        alert('Комната не найдена');
        return;
    }
    const room = await res.json();

    document.getElementById('rooms-view').classList.add('hidden');
    document.getElementById('room-detail').classList.remove('hidden');
    document.getElementById('room-title').textContent = `Комната: ${room.name}`;

    const container = document.getElementById('devices-list');
    container.innerHTML = '';

    if (room.devices.length === 0) {
        container.innerHTML = '<p style="color:#888">Устройств нет. Добавьте первое устройство.</p>';
        return;
    }

    // Fetch full device details
    for (const dev of room.devices) {
        const dres = await fetch(`${API}/rooms/${encodeURIComponent(key)}/devices/${encodeURIComponent(dev.key)}`);
        if (!dres.ok) continue;
        const info = await dres.json();
        const card = document.createElement('div');
        card.className = 'card';

        let details = '';
        if (info.type === 'socket') {
            details = `
                <div class="device-info">
                    <div class="device-info-row">
                        <span class="device-info-label">Состояние:</span>
                        <span class="device-info-value">${info.is_on ? 'Включена' : 'Выключена'}</span>
                    </div>
                    <div class="device-info-row">
                        <span class="device-info-label">Мощность:</span>
                        <span class="device-info-value">${info.power} Вт</span>
                    </div>
                </div>`;
        } else {
            details = `
                <div class="device-info">
                    <div class="device-info-row">
                        <span class="device-info-label">Температура:</span>
                        <span class="device-info-value">${info.temperature}°C</span>
                    </div>
                </div>`;
        }

        const actions = info.type === 'socket'
            ? `<button onclick="toggleSocket('${escapeAttr(key)}','${escapeAttr(dev.key)}', ${info.is_on})">${info.is_on ? 'Выключить' : 'Включить'}</button>`
            : '';

        card.innerHTML = `
            <span class="device-badge ${info.type === 'socket' ? 'badge-socket' : 'badge-thermometer'}">${info.type === 'socket' ? 'Розетка' : 'Термометр'}</span>
            <div class="card-title">${escapeHtml(info.name)}</div>
            <div class="card-subtitle">Ключ: ${escapeHtml(info.key)}</div>
            ${details}
            <div class="card-actions" style="margin-top:12px">
                ${actions}
                <button class="btn-danger" onclick="deleteDevice('${escapeAttr(key)}','${escapeAttr(dev.key)}')">Удалить</button>
            </div>
        `;
        container.appendChild(card);
    }
}

// ---- Devices ---------------------------------------------------------------

function showAddDevice() {
    document.getElementById('device-key').value = '';
    document.getElementById('device-name').value = '';
    document.getElementById('device-power').value = '';
    document.getElementById('device-temp').value = '';
    document.getElementById('device-type').value = 'socket';
    toggleDeviceFields();
    showModal('modal-add-device');
}

function toggleDeviceFields() {
    const type = document.getElementById('device-type').value;
    document.getElementById('socket-fields').classList.toggle('hidden', type !== 'socket');
    document.getElementById('thermometer-fields').classList.toggle('hidden', type !== 'thermometer');
}

async function addDevice() {
    const key = document.getElementById('device-key').value.trim();
    const type = document.getElementById('device-type').value;
    const name = document.getElementById('device-name').value.trim();
    if (!key || !name) return;

    const body = { key, type, name };
    if (type === 'socket') {
        body.is_on = document.getElementById('device-is-on').value === 'true';
        const power = document.getElementById('device-power').value;
        if (power) body.power = parseFloat(power);
    } else {
        const temp = document.getElementById('device-temp').value;
        if (temp) body.temperature = parseFloat(temp);
    }

    const res = await fetch(`${API}/rooms/${encodeURIComponent(currentRoomKey)}/devices`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
    });
    if (res.ok) {
        hideModal('modal-add-device');
        openRoom(currentRoomKey);
    } else {
        const err = await res.json();
        alert(err.error || 'Ошибка');
    }
}

async function deleteDevice(roomKey, devKey) {
    if (!confirm(`Удалить устройство "${devKey}"?`)) return;
    const res = await fetch(`${API}/rooms/${encodeURIComponent(roomKey)}/devices/${encodeURIComponent(devKey)}`, { method: 'DELETE' });
    if (res.ok) openRoom(roomKey);
    else alert('Ошибка удаления');
}

async function toggleSocket(roomKey, devKey, isOn) {
    const action = isOn ? 'turn_off' : 'turn_on';
    const res = await fetch(`${API}/rooms/${encodeURIComponent(roomKey)}/devices/${encodeURIComponent(devKey)}/${action}`, {
        method: 'POST',
    });
    if (res.ok) openRoom(roomKey);
    else alert('Ошибка');
}

// ---- Report ----------------------------------------------------------------

async function getReport() {
    const res = await fetch(`${API}/report`);
    if (!res.ok) {
        alert('Ошибка получения отчёта');
        return;
    }
    const data = await res.json();
    document.getElementById('rooms-view').classList.add('hidden');
    document.getElementById('room-detail').classList.add('hidden');
    document.getElementById('report-view').classList.remove('hidden');
    document.getElementById('report-content').textContent = data.report;
}

// ---- Utils -----------------------------------------------------------------

function showModal(id) {
    document.getElementById(id).classList.remove('hidden');
}

function hideModal(id) {
    document.getElementById(id).classList.add('hidden');
}

function escapeHtml(s) {
    const div = document.createElement('div');
    div.textContent = s;
    return div.innerHTML;
}

function escapeAttr(s) {
    return String(s).replace(/'/g, "\\'");
}

// ---- Init ------------------------------------------------------------------

showRooms();
