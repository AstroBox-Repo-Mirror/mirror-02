// Recovered from the approved 1.3.0 controller; behavior preserved.
function r(e, t) {
  var r = Object.keys(e);
  if (Object.getOwnPropertySymbols) {
    var i = Object.getOwnPropertySymbols(e);
    t && (i = i.filter(function (t) {
      return Object.getOwnPropertyDescriptor(e, t).enumerable;
    })), r.push.apply(r, i);
  }
  return r;
}

function i(e) {
  for (var t = 1; t < arguments.length; t++) {
    var i = null != arguments[t] ? arguments[t] : {};
    t % 2 ? r(Object(i), !0).forEach(function (t) {
      var r, n, a;
      r = e, n = t, a = i[t], (n = function (e) {
        var t = function (e, t) {
          if ("object" != typeof e || !e) return e;
          if ("u" > typeof Symbol && void 0 !== (r = e[Symbol.toPrimitive])) {
            var r,
              i = r.call(e, t || "default");
            if ("object" != typeof i) return i;
            throw TypeError("@@toPrimitive must return a primitive value.");
          }
          return ("string" === t ? String : Number)(e);
        }(e, "string");
        return "symbol" == typeof t ? t : t + "";
      }(n)) in r ? Object.defineProperty(r, n, {
        value: a,
        enumerable: !0,
        configurable: !0,
        writable: !0
      }) : r[n] = a;
    }) : Object.getOwnPropertyDescriptors ? Object.defineProperties(e, Object.getOwnPropertyDescriptors(i)) : r(Object(i)).forEach(function (t) {
      Object.defineProperty(e, t, Object.getOwnPropertyDescriptor(i, t));
    });
  }
  return e;
}

export function createController ({
  send: e,
  persist: t,
  stopAudio: r,
  terminate: n,
  changed: a,
  notify: s = () => {},
  clock: o = () => Date.now()
}) {
  let d = {
      enabled: !0,
      autoPc: !0,
      keepAlive: !1,
      hostId: "",
      ticket: "",
      connected: !1,
      hookReady: !1,
      bridgeEnabled: !1,
      safetyPaused: !1,
      requests: [],
      detail: null,
      hint: "",
      hintId: "",
      status: "连接电脑后同步",
      vibrate: !0,
      fullscreen: !0,
      screen: "inbox",
      transportCode: 0,
      bindingRequired: !1,
      connectionTestText: "测试连接",
      audioValidated: !1,
      audioState: "未运行",
      manualResume: !1,
      firmware: "",
      offset: 0,
      visible: !0,
      probe: {
        startedAt: 0,
        background60s: !1,
        background300s: !1,
        background1800s: !1
      },
      ready: !1
    },
    l = "",
    u = 0,
    c = 0,
    p = 0,
    f = 0,
    b = "",
    y = 0,
    v = 0,
    g = !1,
    m = !0,
    h = new Set(),
    k = new Set(),
    I = new Set(),
    O = new Set(),
    A = () => t({
      bindingRequired: d.bindingRequired,
      enabled: d.enabled,
      autoPc: d.autoPc,
      keepAlive: d.keepAlive,
      hostId: d.hostId,
      vibrate: d.vibrate,
      fullscreen: d.fullscreen
    }),
    P = t => {
      m && e(JSON.stringify(i(i({
        v: 1
      }, t), {}, {
        ticket: d.ticket,
        visible: d.visible
      })));
    },
    _ = () => {
      b && d.connected && d.enabled && d.visible && (P({
        type: "ready",
        nonce: b
      }), b = "");
    },
    w = e => {
      d.visible && (d.hint = e, d.hintId = "ui-" + ++f), a && a(d);
    };
  function S(e) {
    let t = d.requests.find(t => t.id === e);
    t && (d.screen = "detail", d.detail = {
      id: e,
      requestedPage: 0,
      content: "正在加载…",
      page: 0,
      pages: t.pages,
      canApprove: !1,
      kind: t.kind
    }, P({
      type: "detail",
      id: e,
      page: 0
    }), a && a(d));
  }
  function q(e, t = !1) {
    let r = "number" == typeof e.serverTime ? e.serverTime : o() + d.offset,
      i = (e.requests || []).filter(e => !k.has(e.id) && ["approval", "question"].includes(e.kind) && "number" == typeof e.createdAt && r >= e.createdAt && r - e.createdAt <= 15e3);
    (e.requests || []).forEach(e => k.add(e.id));
    let n = new Set(d.requests.map(e => e.id));
    for (let e of k) {
      if (k.size <= 160) break;
      n.has(e) || (k.delete(e), I.delete(e), O.delete(e));
    }
    !t && d.enabled && i.length && (R(i.map(e => e.id)), d.fullscreen && d.visible && "inbox" === d.screen && !d.detail && S(i[0].id));
  }
  function R(e) {
    if (d.vibrate && e.some(e => !I.has(e))) {
      e.forEach(e => I.add(e));
      try {
        s();
      } catch (e) {}
    }
  }
  function x(e) {
    d.visible && d.enabled && Array.isArray(e.alertIds) && R(e.alertIds.filter(e => d.requests.some(t => t.id === e && ["approval", "question"].includes(t.kind)))), d.fullscreen && d.visible && "inbox" === d.screen && !d.detail && "string" == typeof e.wakeRequestId && !O.has(e.wakeRequestId) && d.requests.some(t => t.id === e.wakeRequestId && ["approval", "question"].includes(t.kind)) && (O.add(e.wakeRequestId), R([e.wakeRequestId]), S(e.wakeRequestId));
  }
  function j(e) {
    if ("desktop" !== e.source || h.has(e.id)) return;
    let t = d.visible && d.detail && "approval" === d.detail.kind && d.detail.id === e.id;
    h.add(e.id), t && (d.hint = "allow" === e.state ? "电脑已批准" : "电脑已拒绝", d.hintId = e.id);
  }
  function L(e = !1) {
    if (!g) {
      if (d.bindingRequired) {
        d.status = "电脑绑定已解除", a && a(d);
        return;
      }
      d.ready && d.enabled && (e || d.autoPc) && (d.ticket = "", d.connected = !1, d.detail = null, d.manualResume = e, P({
        type: "hello",
        firmware: d.firmware
      }));
    }
  }
  function T() {
    y = 0, b = "", l = "", u = 0, d.connectionTestText = "测试连接", d.connected = !1, d.hookReady = !1, d.bridgeEnabled = !1, d.ticket = "", d.detail = null, d.requests = [], d.keepAlive = !1, d.status = "未连接本插件电脑", r(), a && a(d);
  }
  return {
    state: d,
    receive: function (e) {
      let t;
      try {
        t = "string" == typeof e ? JSON.parse(e) : e;
      } catch (e) {
        return;
      }
      if (m && !g && d.ready && t && 1 === t.v) {
        if ("hello" === t.type) {
          if (d.bindingRequired || !d.enabled || "string" != typeof t.hostId || "string" != typeof t.ticket || t.ticket.length < 32) return;
          if (d.hostId && d.hostId !== t.hostId) {
            T(), d.status = "电脑身份改变，请先在设置中重新绑定", a && a(d);
            return;
          }
          d.hostId = t.hostId, d.ticket = t.ticket, d.connected = !0, y = 0, d.transportCode = 0, d.hookReady = !0 === t.hookReady, d.bridgeEnabled = !0 === t.enabled, d.safetyPaused = !0 === t.safetyPaused, d.audioValidated = !0 === t.experimentalAudioValidated, d.offset = "number" == typeof t.serverTime ? t.serverTime - o() : 0, d.requests = t.requests || [], d.status = t.enabled ? "已连接电脑" : "电脑桥接已暂停", A(), q(t, !0), "string" == typeof t.launchNonce && t.launchNonce.length >= 32 && (b = t.launchNonce, _()), x(t), !t.enabled && d.manualResume && P({
            type: "resume",
            manual: !0
          }), d.manualResume = !1, a && a(d);
          return;
        }
        if (!d.connected || !d.ticket || t.ticket !== d.ticket) {
          ("unavailable" === t.type || "error" === t.type && "invalid_ticket" === t.error) && T();
          return;
        }
        if (y = 0, "list" === t.type || "resumed" === t.type) d.hookReady = !0 === t.hookReady, d.bridgeEnabled = !1 !== t.enabled, d.safetyPaused = !0 === t.safetyPaused, d.requests = t.requests || [], d.status = "已连接电脑", (t.resolved || []).forEach(j), d.detail && !d.requests.some(e => e.id === d.detail.id) && (d.detail = null), d.detail || "detail" !== d.screen || (d.screen = "inbox"), q(t), x(t), a && a(d);else if ("launch_probe" === t.type) "string" == typeof t.nonce && t.nonce.length >= 32 && (b = t.nonce, _());else if ("detail" === t.type) {
          if (!d.detail || t.id !== d.detail.id || t.page !== d.detail.requestedPage) return;
          d.detail = i(i({}, t), {}, {
            requestedPage: t.page
          }), a && a(d);
        } else if ("result" === t.type) j(t), d.detail && t.id === d.detail.id && (d.detail = null), d.detail || "detail" !== d.screen || (d.screen = "inbox"), d.requests = d.requests.filter(e => e.id !== t.id), d.status = "allow" === t.state ? "已批准本次" : "deny" === t.state ? "已拒绝" : "请求已结束", a && a(d);else if ("pong" === t.type && t.nonce === l) {
          l = "", u = 0, d.connectionTestText = "测试成功", d.status = "互连往返成功", w("连接正常 " + Math.max(0, o() - c) + "ms");
          let e = (o() - v) / 1e3;
          d.probe.startedAt && v && !d.visible && ["保活测试中", "静音播放中"].includes(d.audioState) && [60, 300, 1800].forEach(t => {
            e >= t && e <= t + 30 && (d.probe["background" + t + "s"] = !0);
          }), a && a(d);
        } else "error" === t.type && (d.status = "操作未完成：" + t.error, a && a(d));
      }
    },
    handshake: L,
    disconnect: T,
    transportFailure(e) {
      T(), d.transportCode = "number" == typeof e ? e : 0, a && a(d);
    },
    activate() {
      m = !0, d.ready = !1, d.connected = !1, d.ticket = "", d.detail = null, d.requests = [], d.hint = "", d.hintId = "", h.clear(), k.clear(), I.clear(), O.clear(), d.audioValidated = !1;
    },
    restore(e) {
      if (m) {
        if (d.ready = !0, !e || "object" != typeof e) return void (a && a(d));
        ["enabled", "autoPc", "keepAlive", "vibrate", "fullscreen", "bindingRequired"].forEach(t => {
          "boolean" == typeof e[t] && (d[t] = e[t]);
        }), "string" == typeof e.hostId && (d.hostId = e.hostId), d.keepAlive = !1, a && a(d);
      }
    },
    start() {
      m = !0, d.enabled = !0, A(), L(!0), a && a(d);
    },
    refresh() {
      d.connected ? (y || (y = o() + 15e3), P({
        type: "list"
      })) : L(!0);
    },
    ping: () => d.connected ? !l && (c = o(), u = o() + 6e3, l = String(o()) + "-" + ++p, d.connectionTestText = "正在测试…", a && a(d), P({
      type: "ping",
      nonce: l,
      probeStats: i(i({}, d.probe), {}, {
        firmware: d.firmware,
        visible: d.visible,
        audioState: d.audioState
      })
    }), !0) : (d.connectionTestText = "未连接电脑", w("请先绑定并连接电脑"), !1),
    tick() {
      y && o() >= y && (T(), d.status = "电脑服务无响应", w("电脑无响应，请检查桥接")), l && o() >= u && (l = "", u = 0, d.connectionTestText = "测试超时", w("连接测试超时，请重试"));
    },
    notice: w,
    beginProbe() {
      d.probe = {
        startedAt: o(),
        background60s: !1,
        background300s: !1,
        background1800s: !1
      }, a && a(d);
    },
    visible(e) {
      d.visible && !e && (v = o(), d.probe.background60s = !1, d.probe.background300s = !1, d.probe.background1800s = !1), e && (v = 0), d.visible = e, e && _(), !e && (d.hint = "", d.hintId = "", a && a(d));
    },
    open: S,
    page(e) {
      let t = d.detail;
      t && !(e < 0) && !(e >= t.pages) && (t.requestedPage = e, t.canApprove = !1, P({
        type: "detail",
        id: t.id,
        page: e
      }), a && a(d));
    },
    decide(e) {
      let t = d.detail;
      return !!d.enabled && !!d.connected && !!t && "approval" === t.kind && !!["allow", "deny"].includes(e) && ("number" == typeof t.expiresAt && t.expiresAt > 0 && o() + d.offset >= t.expiresAt ? (d.status = "请求已过期", d.detail = null, a && a(d), !1) : ("allow" !== e || !0 === t.canApprove && t.requestedPage === t.page) && (P({
        type: "decision",
        id: t.id,
        decision: e
      }), d.status = "等待电脑确认…", a && a(d), !0));
    },
    back() {
      d.detail = null, d.screen = "inbox", a && a(d);
    },
    screen(e) {
      ["inbox", "detail", "settings"].includes(e) && (d.screen = e);
    },
    toggleVibrate() {
      d.vibrate = !d.vibrate, A(), a && a(d);
    },
    toggleFullscreen() {
      d.fullscreen = !d.fullscreen, A(), a && a(d);
    },
    toggleAuto() {
      d.autoPc = !d.autoPc, A(), a && a(d);
    },
    forgetHost() {
      if (g) return Promise.resolve(!1);
      let e = d.hostId,
        t = d.bindingRequired;
      d.ticket && P({
        type: "stop"
      }), T(), g = !0, d.hostId = "", d.bindingRequired = !0, d.status = "正在解除电脑绑定", a && a(d);
      let r = () => (g = !1, d.hostId = e, d.bindingRequired = t, d.status = "绑定设置保存失败，请重试", w(d.status), !1);
      try {
        return Promise.resolve(A()).then(e => !1 === e ? r() : (g = !1, d.status = "电脑绑定已解除", w("已解除绑定，请手动绑定"), !0)).catch(r);
      } catch (e) {
        return Promise.resolve(r());
      }
    },
    bindCurrentHost() {
      if (g || d.hostId) return Promise.resolve(!1);
      g = !0, d.bindingRequired = !1, d.status = "正在绑定当前电脑", a && a(d);
      let e = () => (g = !1, d.bindingRequired = !0, w("绑定设置保存失败，请重试"), !1);
      try {
        return Promise.resolve(A()).then(t => !1 === t ? e() : ((g = !1, d.enabled) ? L(!0) : w("请先开启监听"), !0)).catch(e);
      } catch (t) {
        return Promise.resolve(e());
      }
    },
    enableAudio: (e = !1) => d.audioValidated || !0 === e ? m && d.ready && d.connected && d.enabled && d.visible ? (d.keepAlive = !0, d.audioState = "正在启动静音播放", a && a(d), !0) : (w("请先连接电脑"), !1) : (w("请确认后台常驻实验"), !1),
    disableAudio() {
      d.keepAlive = !1, r(), A(), a && a(d);
    },
    stop() {
      let e;
      d.screen = "inbox", d.ticket && P({
        type: "stop"
      }), d.enabled = !1, d.keepAlive = !1;
      try {
        e = A();
      } catch (t) {
        e = Promise.resolve(!1);
      }
      return r(), m = !1, d.connected = !1, d.ticket = "", d.requests = [], d.detail = null, d.hint = "", d.hintId = "", h.clear(), k.clear(), I.clear(), O.clear(), l = "", u = 0, d.probe = {
        startedAt: 0,
        background60s: !1,
        background300s: !1,
        background1800s: !1
      }, d.status = "正在退出并清理缓存", a && a(d), Promise.resolve(e).then(e => {
        if (!1 === e) throw Error("settings_failed");
        return n();
      }).catch(() => {
        d.status = "停用设置保存失败，请重试停止", w(d.status);
      });
    },
    destroy() {
      m = !1, d.ticket = "", d.connected = !1, r();
    }
  };
}
