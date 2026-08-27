<script lang="ts">
    import { onMount } from "svelte";
    import { type Data, defaultData } from "./lib/data";

    let canvas: HTMLCanvasElement;
    let offscreenCanvas = new OffscreenCanvas(1024, 1024);

    let data: Data = $state(defaultData());
    let ws: WebSocket | null = null;
    let isConnected = $state(false);

    let rotateMap = $state(true); // Toggle state for map spinning
    let zoom = $state(1.0); // Zoom level
    let panX = $state(0);   // Camera pan offset X
    let panY = $state(0);   // Camera pan offset Y
    let followedSteamId = $state<string | null>(null); // Custom follow target Steam ID

    let isDragging = false;
    let dragStartX = 0;
    let dragStartY = 0;
    let hasDragged = false;

    onMount(() => {
        const ctx = canvas.getContext("2d")!;
        const offscreen = offscreenCanvas.getContext("2d")!;
        let animId: number;

        const renderLoop = () => {
            render(ctx, offscreen);
            animId = requestAnimationFrame(renderLoop);
        };
        animId = requestAnimationFrame(renderLoop);

        connectWebsocket();

        return () => {
            cancelAnimationFrame(animId);
            if (ws) ws.close();
        };
    });

    function connectWebsocket() {
        const origin = window.location.origin;
        const url = new URL(origin);
        url.protocol = url.protocol === "https:" ? "wss:" : "ws:";
        url.pathname = "client";

        ws = new WebSocket(url);
        
        ws.onopen = () => {
            console.info("Websocket connected");
            isConnected = true;
        };

        ws.onmessage = (event: MessageEvent) => {
            if (typeof event.data !== "string") return;
            try {
                data = JSON.parse(event.data);
            } catch (e) {
                console.error("Failed to parse websocket message", e);
            }
        };

        ws.onclose = () => {
            console.info("Websocket closed, reconnecting in 2s...");
            isConnected = false;
            setTimeout(connectWebsocket, 2000);
        };

        ws.onerror = (err) => {
            console.error("Websocket error", err);
            ws?.close();
        };
    }

    let loadedMap = "";
    let cachedMap: { canvas: OffscreenCanvas, offsetX: number, offsetY: number, width: number, height: number } | null = null;
    const scale = 0.2; // Adjust scale as needed

    $effect(() => {
        if (data.map_name && data.map_name !== loadedMap && data.map_name !== "map_name" && data.map_name !== "<unknown>") {
            loadedMap = data.map_name;
            fetch(`/${loadedMap}.json`)
                .then(r => {
                    if (!r.ok) throw new Error("not found");
                    return r.json();
                })
                .then(lines => {
                    let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
                    for (let i = 0; i < lines.length; i += 4) {
                        const x1 = lines[i] * scale, y1 = -lines[i+1] * scale;
                        const x2 = lines[i+2] * scale, y2 = -lines[i+3] * scale;
                        if (x1 < minX) minX = x1; if (x1 > maxX) maxX = x1;
                        if (y1 < minY) minY = y1; if (y1 > maxY) maxY = y1;
                        if (x2 < minX) minX = x2; if (x2 > maxX) maxX = x2;
                        if (y2 < minY) minY = y2; if (y2 > maxY) maxY = y2;
                    }
                    
                    const padding = 20;
                    const width = maxX - minX + padding * 2;
                    const height = maxY - minY + padding * 2;
                    
                    const mapCanvas = new OffscreenCanvas(Math.ceil(width), Math.ceil(height));
                    const mapCtx = mapCanvas.getContext("2d")!;
                    mapCtx.translate(-minX + padding, -minY + padding);
                    
                    const path = new Path2D();
                    for (let i = 0; i < lines.length; i += 4) {
                        path.moveTo(lines[i] * scale, -lines[i+1] * scale);
                        path.lineTo(lines[i+2] * scale, -lines[i+3] * scale);
                    }
                    
                    // Pass 1: Dark outer glow outline for high contrast legibility
                    mapCtx.strokeStyle = "rgba(0, 0, 0, 0.9)";
                    mapCtx.lineWidth = 3.5;
                    mapCtx.lineCap = "round";
                    mapCtx.stroke(path);

                    // Pass 2: High contrast crisp map lines
                    mapCtx.strokeStyle = "rgba(220, 240, 255, 0.75)";
                    mapCtx.lineWidth = 1.5;
                    mapCtx.stroke(path);

                    cachedMap = {
                        canvas: mapCanvas,
                        offsetX: minX - padding,
                        offsetY: minY - padding,
                        width,
                        height
                    };
                })
                .catch(e => {
                    console.error("Failed to load map geometry", e);
                    cachedMap = null;
                });
        }
    });

    let killFeed: Array<{ id: number, killer: string, weapon: string, time: number }> = $state([]);
    let nextKillId = 0;
    let prevKills: Record<string, number> = {};

    $effect(() => {
        if (data.in_game) {
            const allPlayers = [...(data.friendlies || []), ...(data.players || []), ...(data.local_player ? [data.local_player] : [])];
            
            for (const p of allPlayers) {
                const id = p.steam_id.toString();
                if (prevKills[id] !== undefined && p.round_kills > prevKills[id]) {
                    const weaponName = (typeof p.weapon === 'string' ? p.weapon : (p.weapon?.name || "Weapon"));
                    killFeed = [...killFeed, {
                        id: nextKillId++,
                        killer: p.name || "Unknown",
                        weapon: weaponName,
                        time: Date.now()
                    }];
                    if (killFeed.length > 5) killFeed.shift();
                }
                prevKills[id] = p.round_kills;
            }
            
            const now = Date.now();
            killFeed = killFeed.filter(k => now - k.time < 5000);
        }
    });

    function getFollowedPlayer(): any {
        if (!data.in_game) return null;
        const allPlayers = [
            ...(data.local_player ? [data.local_player] : []),
            ...(data.friendlies || []),
            ...(data.players || [])
        ];
        if (followedSteamId) {
            const found = allPlayers.find(p => p.steam_id.toString() === followedSteamId);
            if (found) return found;
        }
        return data.local_player || allPlayers[0] || null;
    }

    function onPointerDown(e: PointerEvent) {
        isDragging = true;
        dragStartX = e.clientX;
        dragStartY = e.clientY;
        hasDragged = false;
        (e.target as HTMLElement).setPointerCapture(e.pointerId);
    }

    function onPointerMove(e: PointerEvent) {
        if (!isDragging) return;
        const dx = e.clientX - dragStartX;
        const dy = e.clientY - dragStartY;
        if (Math.hypot(dx, dy) > 4) {
            hasDragged = true;
        }
        panX += dx / zoom;
        panY += dy / zoom;
        dragStartX = e.clientX;
        dragStartY = e.clientY;
    }

    function onPointerUp(e: PointerEvent) {
        if (isDragging) {
            isDragging = false;
            try {
                (e.target as HTMLElement).releasePointerCapture(e.pointerId);
            } catch (_) {}
        }
    }

    function onWheel(e: WheelEvent) {
        e.preventDefault();
        const factor = e.deltaY < 0 ? 1.15 : 0.87;
        zoom = Math.max(0.2, Math.min(5.0, zoom * factor));
    }

    function onCanvasClick(e: MouseEvent) {
        if (hasDragged) return;
        const rect = canvas.getBoundingClientRect();
        const clickX = (e.clientX - rect.left) * window.devicePixelRatio;
        const clickY = (e.clientY - rect.top) * window.devicePixelRatio;

        const dpr = window.devicePixelRatio;
        const centerX = canvas.width / 2 + panX * dpr;
        const centerY = canvas.height / 2 + panY * dpr;

        const allPlayers = [
            ...(data.local_player ? [data.local_player] : []),
            ...(data.friendlies || []),
            ...(data.players || [])
        ];

        const targetPlayer = getFollowedPlayer();
        let canvasRotation = 0;
        if (rotateMap && targetPlayer) {
            let yaw = 0;
            if (targetPlayer === data.local_player && data.view_angles && data.view_angles.length >= 2) {
                yaw = data.view_angles[1];
            } else if (targetPlayer.rotation !== undefined) {
                yaw = targetPlayer.rotation;
            }
            canvasRotation = (yaw - 90) * Math.PI / 180;
        }

        let targetX = 0, targetY = 0;
        if (targetPlayer && targetPlayer.position && targetPlayer.position.length >= 2) {
            targetX = targetPlayer.position[0] * scale;
            targetY = -targetPlayer.position[1] * scale;
        }

        let closestPlayer: any = null;
        let closestDist = 35 * dpr;

        for (const p of allPlayers) {
            if (!p.position || p.position.length < 2) continue;
            const px = p.position[0] * scale;
            const py = -p.position[1] * scale;

            let dx = px - targetX;
            let dy = py - targetY;
            if (canvasRotation !== 0) {
                const cos = Math.cos(canvasRotation);
                const sin = Math.sin(canvasRotation);
                const rx = dx * cos - dy * sin;
                const ry = dx * sin + dy * cos;
                dx = rx;
                dy = ry;
            }

            const screenX = centerX + dx * zoom * dpr;
            const screenY = centerY + dy * zoom * dpr;

            const dist = Math.hypot(clickX - screenX, clickY - screenY);
            if (dist < closestDist) {
                closestDist = dist;
                closestPlayer = p;
            }
        }

        if (closestPlayer) {
            followedSteamId = closestPlayer.steam_id.toString();
        }
    }

    function render(ctx: CanvasRenderingContext2D, offscreen: OffscreenCanvasRenderingContext2D) {
        const dpr = window.devicePixelRatio;
        canvas.width = canvas.clientWidth * dpr;
        canvas.height = canvas.clientHeight * dpr;
        
        // Dark tactical background gradient
        const bgGrad = ctx.createRadialGradient(
            canvas.width / 2, canvas.height / 2, 50,
            canvas.width / 2, canvas.height / 2, Math.max(canvas.width, canvas.height)
        );
        bgGrad.addColorStop(0, "#0b0f19");
        bgGrad.addColorStop(1, "#05070c");
        ctx.fillStyle = bgGrad;
        ctx.fillRect(0, 0, canvas.width, canvas.height);

        // Draw faint background radar grid lines
        ctx.save();
        ctx.strokeStyle = "rgba(0, 229, 255, 0.04)";
        ctx.lineWidth = 1;
        const gridSize = 80 * dpr * zoom;
        const offsetX = (canvas.width / 2 + panX * dpr) % gridSize;
        const offsetY = (canvas.height / 2 + panY * dpr) % gridSize;

        for (let x = offsetX; x < canvas.width; x += gridSize) {
            ctx.beginPath();
            ctx.moveTo(x, 0);
            ctx.lineTo(x, canvas.height);
            ctx.stroke();
        }
        for (let y = offsetY; y < canvas.height; y += gridSize) {
            ctx.beginPath();
            ctx.moveTo(0, y);
            ctx.lineTo(canvas.width, y);
            ctx.stroke();
        }
        ctx.restore();

        const targetPlayer = getFollowedPlayer();

        ctx.save();
        ctx.translate(canvas.width / 2 + panX * dpr, canvas.height / 2 + panY * dpr);
        ctx.scale(zoom, zoom);

        let canvasRotation = 0;
        if (rotateMap && targetPlayer) {
            let yaw = 0;
            if (targetPlayer === data.local_player && data.view_angles && data.view_angles.length >= 2) {
                yaw = data.view_angles[1];
            } else if (targetPlayer.rotation !== undefined) {
                yaw = targetPlayer.rotation;
            }
            canvasRotation = (yaw - 90) * Math.PI / 180;
            ctx.rotate(canvasRotation);
        }

        if (targetPlayer && targetPlayer.position && targetPlayer.position.length >= 2) {
            ctx.translate(-targetPlayer.position[0] * scale, targetPlayer.position[1] * scale);
        }

        if (cachedMap) {
            ctx.drawImage(cachedMap.canvas, cachedMap.offsetX, cachedMap.offsetY);
        }

        const drawPlayer = (p: any, color: string, isEnemy: boolean) => {
            if (!p || !p.position || p.position.length < 2) return;
            const x = p.position[0] * scale;
            const y = -p.position[1] * scale;
            const isFollowed = targetPlayer && p.steam_id.toString() === targetPlayer.steam_id.toString();
            
            ctx.save();
            ctx.translate(x, y);

            // Draw glowing aura around followed player
            if (isFollowed) {
                ctx.beginPath();
                ctx.arc(0, 0, 16, 0, Math.PI * 2);
                ctx.fillStyle = "rgba(0, 240, 255, 0.2)";
                ctx.fill();
                ctx.strokeStyle = "#00f0ff";
                ctx.lineWidth = 2;
                ctx.shadowColor = "#00f0ff";
                ctx.shadowBlur = 10;
                ctx.stroke();
                ctx.shadowBlur = 0;
            }

            let wname = "";
            if (p.weapon) {
                wname = (typeof p.weapon === 'string' ? p.weapon : (p.weapon.name || "")).toLowerCase();
            }
            const isSniper = wname.includes("awp") || wname.includes("ssg") || wname.includes("sgg") || wname.includes("scout");

            // View Direction Cone / Line
            if (p.rotation !== undefined) {
                const viewYaw = p.rotation;
                const rad = viewYaw * Math.PI / 180;

                if (isSniper) {
                    const dirX = Math.cos(rad) * 1200;
                    const dirY = -Math.sin(rad) * 1200;
                    const gradient = ctx.createLinearGradient(0, 0, dirX, dirY);
                    gradient.addColorStop(0, color);
                    gradient.addColorStop(1, "rgba(0, 0, 0, 0)");

                    ctx.beginPath();
                    ctx.moveTo(0, 0);
                    ctx.lineTo(dirX, dirY);
                    ctx.strokeStyle = gradient;
                    ctx.lineWidth = 2;
                    ctx.stroke();
                } else {
                    // Vision Cone
                    const coneLen = 45;
                    const fovHalf = 30 * Math.PI / 180;
                    const leftRad = rad - fovHalf;
                    const rightRad = rad + fovHalf;

                    ctx.beginPath();
                    ctx.moveTo(0, 0);
                    ctx.lineTo(Math.cos(leftRad) * coneLen, -Math.sin(leftRad) * coneLen);
                    ctx.arc(0, 0, coneLen, -leftRad, -rightRad, true);
                    ctx.closePath();

                    const coneGrad = ctx.createRadialGradient(0, 0, 0, 0, 0, coneLen);
                    coneGrad.addColorStop(0, isEnemy ? "rgba(255, 42, 95, 0.25)" : "rgba(0, 229, 255, 0.2)");
                    coneGrad.addColorStop(1, "rgba(0, 0, 0, 0)");
                    ctx.fillStyle = coneGrad;
                    ctx.fill();
                }
            }

            // Outer ring & player dot
            ctx.beginPath();
            ctx.arc(0, 0, 7, 0, Math.PI * 2);
            ctx.fillStyle = color;
            ctx.fill();
            ctx.strokeStyle = "#000000";
            ctx.lineWidth = 1.5;
            ctx.stroke();

            ctx.rotate(-canvasRotation);

            if (p.has_bomb) {
                ctx.font = "14px sans-serif";
                ctx.textAlign = "center";
                ctx.fillText("💣", 0, -34);
            }

            // High Legibility Text Badges
            const nameText = p.name || "Unknown";
            ctx.font = isFollowed ? "bold 13px 'Inter', sans-serif" : "500 12px 'Inter', sans-serif";
            
            // Draw background stroke for maximum contrast legibility
            ctx.strokeStyle = "#000000";
            ctx.lineWidth = 4;
            ctx.lineJoin = "round";
            ctx.textAlign = "center";
            ctx.strokeText(nameText, 0, -20);
            ctx.fillStyle = "#ffffff";
            ctx.fillText(nameText, 0, -20);

            if (p.weapon) {
                const wname = typeof p.weapon === 'string' ? p.weapon : (p.weapon.name || "");
                if (wname) {
                    let wtext = wname;
                    if (p.ammo && p.ammo.length >= 2 && p.ammo[0] >= 0) {
                        wtext += ` (${p.ammo[0]}/${p.ammo[1]})`;
                    }
                    ctx.font = "10px sans-serif";
                    ctx.strokeStyle = "#000000";
                    ctx.lineWidth = 3;
                    ctx.strokeText(wtext, 0, 22);
                    ctx.fillStyle = "#aaccff";
                    ctx.fillText(wtext, 0, 22);
                }
            }
            
            // Health & Armor Bar
            if (p.health !== undefined) {
                const hpWidth = 24;
                const hpPct = Math.max(0, Math.min(100, p.health)) / 100;
                
                // HP Bar BG
                ctx.fillStyle = "rgba(0, 0, 0, 0.8)";
                ctx.fillRect(-hpWidth/2 - 1, -15, hpWidth + 2, 5);
                
                // HP Bar Fill
                const r = Math.floor(255 * (1 - hpPct));
                const g = Math.floor(255 * hpPct);
                ctx.fillStyle = `rgb(${r}, ${g}, 50)`;
                ctx.fillRect(-hpWidth/2, -14, hpWidth * hpPct, 3);
            }

            if (p.armor !== undefined && p.armor > 0) {
                const arWidth = 24;
                const arPct = Math.max(0, Math.min(100, p.armor)) / 100;
                ctx.fillStyle = "rgba(0, 0, 0, 0.8)";
                ctx.fillRect(-arWidth/2 - 1, -9, arWidth + 2, 3);
                ctx.fillStyle = "#00d5ff";
                ctx.fillRect(-arWidth/2, -8, arWidth * arPct, 1.5);
            }
            
            ctx.restore();
        };

        // Draw Bomb
        if (data.bomb && data.bomb.planted && data.bomb.position && data.bomb.position.length >= 2) {
            const bx = data.bomb.position[0] * scale;
            const by = -data.bomb.position[1] * scale;
            
            ctx.save();
            ctx.translate(bx, by);

            const pulse = (Date.now() % 1000) / 1000;
            ctx.beginPath();
            ctx.arc(0, 0, 10 + pulse * 8, 0, Math.PI * 2);
            ctx.fillStyle = `rgba(255, 0, 0, ${0.4 - pulse * 0.4})`;
            ctx.fill();

            ctx.beginPath();
            ctx.arc(0, 0, 8, 0, Math.PI * 2);
            ctx.fillStyle = (Date.now() % 600 < 300) ? "#ff1a1a" : "#ff9900";
            ctx.fill();
            ctx.strokeStyle = "#ffffff";
            ctx.lineWidth = 1.5;
            ctx.stroke();
            
            ctx.rotate(-canvasRotation);
            const timerText = data.bomb.timer ? data.bomb.timer.toFixed(1) + "s" : "C4";
            ctx.font = "bold 13px sans-serif";
            ctx.textAlign = "center";
            ctx.strokeStyle = "#000000";
            ctx.lineWidth = 4;
            ctx.strokeText(timerText, 0, -14);
            ctx.fillStyle = "#ffdd00";
            ctx.fillText(timerText, 0, -14);
            
            ctx.restore();
        }

        // Item / Entity ESP
        if (data.item_esp_active && data.entities) {
            data.entities.forEach(e => {
                let pos;
                let name = "";
                let color = "#ffffff";
                if (e.Weapon) {
                    pos = e.Weapon.position;
                    const wname = typeof e.Weapon.weapon === 'string' ? e.Weapon.weapon : (e.Weapon.weapon.name || "");
                    name = wname;
                    if (e.Weapon.ammo && e.Weapon.ammo.length === 2 && e.Weapon.ammo[0] >= 0) {
                        name += ` (${e.Weapon.ammo[0]}/${e.Weapon.ammo[1]})`;
                    }
                    color = "#99ccff";
                } else if (e.Smoke) {
                    pos = e.Smoke.position;
                    name = "Smoke";
                    color = "#cccccc";
                } else if (e.Inferno) {
                    pos = e.Inferno.position;
                    name = "Molotov";
                    color = "#ff8800";
                } else if (e.Molotov) {
                    pos = e.Molotov.position;
                    name = e.Molotov.is_incendiary ? "Incendiary" : "Molotov";
                    color = "#ff8800";
                } else if (e.Flashbang) {
                    pos = e.Flashbang.position;
                    name = "Flashbang";
                    color = "#ffff88";
                } else if (e.HeGrenade) {
                    pos = e.HeGrenade.position;
                    name = "HE Grenade";
                    color = "#ff4444";
                } else if (e.Decoy) {
                    pos = e.Decoy.position;
                    name = "Decoy";
                    color = "#ffffff";
                }

                if (pos && pos.length >= 2) {
                    const x = pos[0] * scale;
                    const y = -pos[1] * scale;
                    
                    ctx.save();
                    ctx.translate(x, y);
                    ctx.rotate(-canvasRotation);
                    
                    ctx.font = "500 11px sans-serif";
                    ctx.textAlign = "center";
                    ctx.strokeStyle = "#000000";
                    ctx.lineWidth = 3;
                    ctx.strokeText(name, 0, 0);
                    ctx.fillStyle = color;
                    ctx.fillText(name, 0, 0);
                    ctx.restore();
                }
            });
        }

        // Draw players in order
        if (data.friendlies) {
            data.friendlies.forEach(p => drawPlayer(p, "#00ff88", false));
        }
        if (data.local_player) {
            drawPlayer(data.local_player, "#00e5ff", false);
        }
        if (data.players) {
            data.players.forEach(p => drawPlayer(p, "#ff2a5f", true));
        }
        
        ctx.restore();
    }
</script>

<div class="hud-top-bar">
    <div class="brand">
        <span class="logo-dot"></span>
        DEADLOCKED <span class="radar-tag">RADAR</span>
    </div>
    <div class="map-badge">{data.map_name ?? "Map Unknown"}</div>
    <div class="status-badge" class:connected={isConnected}>
        <span class="status-dot"></span>
        {isConnected ? (data.in_game ? "LIVE MATCH" : "CONNECTED") : "RECONNECTING"}
    </div>
</div>

{#if data.in_game}
<div class="scoreboard">
    <div class="scoreboard-title">PLAYERS IN MATCH</div>
    {#if data.local_player}
        <button 
            type="button"
            class="player-row local"
            class:has-c4={data.local_player.has_bomb}
            class:following={!followedSteamId || followedSteamId === data.local_player.steam_id.toString()}
            onclick={() => followedSteamId = data.local_player.steam_id.toString()}
        >
            <div class="player-info">
                <div class="player-name">{data.local_player.name || "Local Player"} (You)</div>
                <div class="player-weapon">
                    {#if typeof data.local_player.weapon === 'string'}
                        {data.local_player.weapon}
                    {:else if data.local_player.weapon && data.local_player.weapon.name}
                        {data.local_player.weapon.name} {data.local_player.ammo && data.local_player.ammo[0] >= 0 ? `(${data.local_player.ammo[0]}/${data.local_player.ammo[1]})` : ""}
                    {/if}
                </div>
            </div>
            <div class="player-hp">{data.local_player.health} <span class="hp-label">HP</span></div>
        </button>
    {/if}
    {#each (data.friendlies || []) as p}
        <button 
            type="button"
            class="player-row friendly" 
            class:has-c4={p.has_bomb}
            class:following={followedSteamId === p.steam_id.toString()}
            onclick={() => followedSteamId = p.steam_id.toString()}
        >
            <div class="player-info">
                <div class="player-name">{p.name || "Teammate"}</div>
                <div class="player-weapon">
                    {#if typeof p.weapon === 'string'}
                        {p.weapon}
                    {:else if p.weapon && p.weapon.name}
                        {p.weapon.name} {p.ammo && p.ammo[0] >= 0 ? `(${p.ammo[0]}/${p.ammo[1]})` : ""}
                    {/if}
                </div>
            </div>
            <div class="player-hp">{p.health} <span class="hp-label">HP</span></div>
        </button>
    {/each}
    {#each (data.players || []) as p}
        <button 
            type="button"
            class="player-row enemy" 
            class:has-c4={p.has_bomb}
            class:following={followedSteamId === p.steam_id.toString()}
            onclick={() => followedSteamId = p.steam_id.toString()}
        >
            <div class="player-info">
                <div class="player-name">{p.name || "Enemy"}</div>
                <div class="player-weapon">
                    {#if typeof p.weapon === 'string'}
                        {p.weapon}
                    {:else if p.weapon && p.weapon.name}
                        {p.weapon.name} {p.ammo && p.ammo[0] >= 0 ? `(${p.ammo[0]}/${p.ammo[1]})` : ""}
                    {/if}
                </div>
            </div>
            <div class="player-hp">{p.health} <span class="hp-label">HP</span></div>
        </button>
    {/each}
</div>
{/if}

<div class="controls">
    <label class="toggle-container">
        <input type="checkbox" bind:checked={rotateMap} />
        <span class="toggle-slider"></span>
        Spin Map
    </label>

    <div class="control-divider"></div>

    <label for="follow-select" class="select-label">Spectate:</label>
    <select id="follow-select" bind:value={followedSteamId}>
        {#if data.local_player}
            <option value={data.local_player.steam_id.toString()}>{data.local_player.name || "Local Player"} (You)</option>
        {/if}
        {#if data.friendlies && data.friendlies.length > 0}
            <optgroup label="Teammates">
                {#each data.friendlies as p}
                    <option value={p.steam_id.toString()}>{p.name || "Teammate"}</option>
                {/each}
            </optgroup>
        {/if}
        {#if data.players && data.players.length > 0}
            <optgroup label="Enemies">
                {#each data.players as p}
                    <option value={p.steam_id.toString()}>{p.name || "Enemy"}</option>
                {/each}
            </optgroup>
        {/if}
    </select>

    <div class="control-divider"></div>

    <div class="btn-group">
        <button type="button" class="ctrl-btn" onclick={() => zoom = Math.min(5.0, zoom * 1.25)} title="Zoom In">+</button>
        <button type="button" class="ctrl-btn" onclick={() => zoom = Math.max(0.2, zoom / 1.25)} title="Zoom Out">-</button>
        <button type="button" class="ctrl-btn reset-btn" onclick={() => { panX = 0; panY = 0; zoom = 1.0; }}>Reset View</button>
    </div>
</div>

<div class="killfeed">
    {#each killFeed as kill (kill.id)}
        <div class="kill-event">
            <span class="killer">{kill.killer}</span>
            <span class="weapon">[{kill.weapon}]</span>
        </div>
    {/each}
</div>

<canvas 
    bind:this={canvas}
    onpointerdown={onPointerDown}
    onpointermove={onPointerMove}
    onpointerup={onPointerUp}
    onpointercancel={onPointerUp}
    onwheel={onWheel}
    onclick={onCanvasClick}
></canvas>

<style>
    @import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap');

    :global(body) {
        margin: 0;
        padding: 0;
        background: #05070c;
        overflow: hidden;
        font-family: 'Inter', system-ui, -apple-system, sans-serif;
        user-select: none;
    }

    .hud-top-bar {
        position: absolute;
        top: 1rem;
        left: 1rem;
        display: flex;
        align-items: center;
        gap: 12px;
        z-index: 10;
    }

    .brand {
        background: rgba(12, 16, 26, 0.85);
        backdrop-filter: blur(12px);
        border: 1px solid rgba(255, 255, 255, 0.12);
        padding: 8px 14px;
        border-radius: 10px;
        color: #ffffff;
        font-size: 13px;
        font-weight: 700;
        letter-spacing: 1px;
        display: flex;
        align-items: center;
        gap: 8px;
        box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
    }

    .logo-dot {
        width: 8px;
        height: 8px;
        background: #00e5ff;
        border-radius: 50%;
        box-shadow: 0 0 8px #00e5ff;
    }

    .radar-tag {
        color: #00e5ff;
    }

    .map-badge {
        background: rgba(12, 16, 26, 0.85);
        backdrop-filter: blur(12px);
        border: 1px solid rgba(255, 255, 255, 0.12);
        padding: 8px 14px;
        border-radius: 10px;
        color: #e0e6ed;
        font-size: 12px;
        font-weight: 600;
        box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
    }

    .status-badge {
        background: rgba(12, 16, 26, 0.85);
        backdrop-filter: blur(12px);
        border: 1px solid rgba(255, 255, 255, 0.12);
        padding: 8px 14px;
        border-radius: 10px;
        color: #ff9900;
        font-size: 11px;
        font-weight: 600;
        display: flex;
        align-items: center;
        gap: 6px;
        box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
    }

    .status-badge.connected {
        color: #00ff88;
    }

    .status-dot {
        width: 6px;
        height: 6px;
        border-radius: 50%;
        background: currentColor;
        box-shadow: 0 0 6px currentColor;
    }

    .scoreboard {
        position: absolute;
        top: 4.2rem;
        left: 1rem;
        background: rgba(10, 14, 24, 0.88);
        backdrop-filter: blur(16px);
        border: 1px solid rgba(255, 255, 255, 0.12);
        padding: 12px;
        border-radius: 12px;
        color: white;
        display: flex;
        flex-direction: column;
        gap: 6px;
        max-height: 72vh;
        width: 240px;
        overflow-y: auto;
        z-index: 10;
        box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
    }

    .scoreboard-title {
        font-size: 10px;
        font-weight: 700;
        color: #8a9bb0;
        letter-spacing: 1px;
        margin-bottom: 4px;
        padding-left: 4px;
    }

    .player-row {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 8px 10px;
        border-radius: 8px;
        background: rgba(255, 255, 255, 0.04);
        border: 1px solid rgba(255, 255, 255, 0.06);
        color: white;
        cursor: pointer;
        width: 100%;
        text-align: left;
        transition: all 0.15s ease;
    }

    .player-row:hover {
        background: rgba(255, 255, 255, 0.12);
        border-color: rgba(255, 255, 255, 0.2);
    }

    .player-info {
        display: flex;
        flex-direction: column;
        gap: 2px;
        overflow: hidden;
    }

    .player-name {
        font-size: 12px;
        font-weight: 600;
        color: #f0f4f8;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .player-weapon {
        font-size: 10px;
        color: #8a9bb0;
    }

    .player-hp {
        font-size: 12px;
        font-weight: 700;
        color: #00ff88;
    }

    .hp-label {
        font-size: 9px;
        font-weight: 500;
        color: #8a9bb0;
    }

    .player-row.local {
        border-left: 4px solid #00e5ff;
    }

    .player-row.friendly {
        border-left: 4px solid #00ff88;
    }

    .player-row.enemy {
        border-left: 4px solid #ff2a5f;
    }

    .player-row.following {
        box-shadow: 0 0 0 2px #00f0ff, 0 0 12px rgba(0, 240, 255, 0.4);
        background: rgba(0, 240, 255, 0.15) !important;
    }

    .killfeed {
        position: absolute;
        top: 1rem;
        right: 1rem;
        display: flex;
        flex-direction: column;
        gap: 6px;
        align-items: flex-end;
        z-index: 10;
    }

    .kill-event {
        background: rgba(10, 14, 24, 0.88);
        backdrop-filter: blur(12px);
        border: 1px solid rgba(255, 255, 255, 0.12);
        padding: 6px 12px;
        border-radius: 8px;
        color: white;
        font-size: 12px;
        display: flex;
        gap: 8px;
        animation: fadein 0.25s ease-out;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
    }

    @keyframes fadein {
        from { opacity: 0; transform: translateX(20px); }
        to { opacity: 1; transform: translateX(0); }
    }

    .killer { font-weight: 700; color: #00e5ff; }
    .weapon { color: #8a9bb0; font-weight: 500; }
    
    .controls {
        position: absolute;
        bottom: 1.5rem;
        left: 50%;
        transform: translateX(-50%);
        background: rgba(10, 14, 24, 0.88);
        backdrop-filter: blur(16px);
        border: 1px solid rgba(255, 255, 255, 0.15);
        padding: 8px 16px;
        border-radius: 14px;
        color: white;
        display: flex;
        align-items: center;
        gap: 14px;
        z-index: 10;
        box-shadow: 0 8px 32px rgba(0, 0, 0, 0.6);
    }

    .toggle-container {
        display: flex;
        align-items: center;
        gap: 8px;
        cursor: pointer;
        font-size: 12px;
        font-weight: 600;
        color: #d0dbe5;
    }

    .select-label {
        font-size: 12px;
        font-weight: 600;
        color: #8a9bb0;
    }

    .controls select {
        background: rgba(255, 255, 255, 0.08);
        color: #f0f4f8;
        border: 1px solid rgba(255, 255, 255, 0.2);
        border-radius: 6px;
        padding: 5px 10px;
        font-size: 12px;
        font-weight: 500;
        cursor: pointer;
        outline: none;
    }

    .controls select option {
        background: #0f1420;
        color: white;
    }

    .control-divider {
        width: 1px;
        height: 22px;
        background: rgba(255, 255, 255, 0.15);
    }

    .btn-group {
        display: flex;
        gap: 6px;
    }

    .ctrl-btn {
        background: rgba(255, 255, 255, 0.1);
        color: white;
        border: 1px solid rgba(255, 255, 255, 0.18);
        padding: 5px 12px;
        border-radius: 6px;
        cursor: pointer;
        font-size: 12px;
        font-weight: 700;
        transition: all 0.15s ease;
    }

    .ctrl-btn:hover {
        background: rgba(0, 229, 255, 0.2);
        border-color: #00e5ff;
        color: #00e5ff;
    }

    .reset-btn {
        font-weight: 600;
    }

    canvas {
        width: 100dvw;
        height: 100dvh;
        position: absolute;
        top: 0;
        left: 0;
        z-index: 1;
        cursor: grab;
        touch-action: none;
    }

    canvas:active {
        cursor: grabbing;
    }
</style>
