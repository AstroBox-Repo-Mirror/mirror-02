import * as NativeTransport from './native_transport.js';
import * as Flags from './build_flags.js';
import * as Controller from './controller.js';
import * as SimulatedHost from './simulated_host.js';
import * as SimulatorTransport from './simulator_transport.js';
import system0 from '@system.interconnect';
import system1 from '@system.storage';
import system2 from '@system.app';
import system3 from '@system.device';
import system4 from '@system.vibrator';
const serviceExports={};
let i, n;
Object.defineProperty(serviceExports, "__esModule", {
  value: !0
}), Object.defineProperty(serviceExports, "PROBE_BUILD", {
  enumerable: !0,
  get: function () {
    return y.PROBE_BUILD;
  }
}), Object.defineProperty(serviceExports, "SIMULATOR_BUILD", {
  enumerable: !0,
  get: function () {
    return y.SIMULATOR_BUILD;
  }
}), serviceExports.checkAudio = function () {}, serviceExports.controller = void 0, serviceExports.destroy = function () {
  P.destroy(), S(), k = [];
}, serviceExports.initialize = q, serviceExports.simulationAction = function (e) {
  if (y.SIMULATOR_BUILD && !y.LIVE_SIMULATOR_BUILD) {
    if ("new" === e && A.newRequest(), "expire" === e && A.expire(), "disconnect" === e) {
      A.disconnect(), P.disconnect();
      return;
    }
    if ("reconnect" === e) {
      A.reconnect(), P.handshake(!0);
      return;
    }
    P.refresh();
  }
}, serviceExports.start = function () {
  if (I || q(), !P.state.ready) {
    O = !0;
    return;
  }
  P.start();
}, serviceExports.startAudio = function () {
  return P.notice("当前手环不支持音频常驻"), !1;
}, serviceExports.subscribe = function (e) {
  return k.push(e), e(P.state), () => {
    k = k.filter(t => t !== e);
  };
};
var l = {
    default: system0
  },
  u = {
    default: system1
  },
  c = {
    default: system2
  },
  p = {
    default: system3
  },
  f = {
    default: system4
  },
  b = Controller,
  y = Flags,
  v = SimulatorTransport,
  g = SimulatedHost,
  m = NativeTransport;
function h(e) {
  return e && e.__esModule ? e : {
    default: e
  };
}
let k = [],
  I = !1,
  O = !1,
  A = y.SIMULATOR_BUILD && !y.LIVE_SIMULATOR_BUILD ? (0, g.createSimulatedHost)() : null,
  P = serviceExports.controller = (0, b.createController)({
    send: e => {
      if (y.LIVE_SIMULATOR_BUILD) (0, v.sendToLiveSimulator)(e, e => P.receive(e), () => P.disconnect());else if (y.SIMULATOR_BUILD) {
        let t = A.receive(e);
        setTimeout(() => P.receive(t), 30);
      } else i && (0, m.sendNative)(i, e, e => P.transportFailure(e));
    },
    persist: e => new Promise(t => u.default.set({
      key: "codex-band-settings",
      value: JSON.stringify(e),
      success: () => t(!0),
      fail: () => t(!1)
    })),
    stopAudio: _,
    notify: () => {
      if (!y.SIMULATOR_BUILD) try {
        (!c.default.canIUse || c.default.canIUse("@system.vibrator.vibrate")) && f.default.vibrate({
          mode: "long"
        });
      } catch (e) {}
    },
    terminate: () => (S(), new Promise((e, t) => setTimeout(() => {
      try {
        c.default.terminate(), e();
      } catch (e) {
        t(e);
      }
    }, 150))),
    changed: e => k.forEach(t => t(e))
  });
function _() {
  P.state.keepAlive = !1, P.state.audioState = "当前固件不支持";
}
function w() {
  n || (n = setInterval(() => {
    P.tick(), (0, m.refreshTransport)(P);
  }, y.LIVE_SIMULATOR_BUILD ? 2e3 : 5e3));
}
function S() {
  n && (clearInterval(n), n = null), i && (i.onmessage = null, i.onopen = null, i.onclose = null, i.onerror = null), _(), I = !1;
}
function q() {
  if (I) return;
  I = !0, P.activate(), y.SIMULATOR_BUILD || ((i = l.default.instance()).onmessage = e => P.receive(e.data), i.onopen = () => {
    w(), P.handshake();
  }, i.onclose = () => {
    P.disconnect(), w();
  }, i.onerror = e => P.transportFailure(e && e.code)), p.default.getInfo({
    success: e => {
      P.state.firmware = String(e.osVersionName || ""), P.handshake();
    }
  });
  let e = () => {
    O ? (O = !1, P.start()) : P.handshake();
  };
  u.default.get({
    key: "codex-band-settings",
    success: t => {
      try {
        P.restore(JSON.parse(t));
      } catch (e) {
        P.restore({});
      }
      e();
    },
    fail: () => {
      P.restore({}), e();
    }
  }), w();
}
export const controller=serviceExports.controller;
export const initialize=serviceExports.initialize;
export const destroy=serviceExports.destroy;
export const subscribe=serviceExports.subscribe;
export const start=serviceExports.start;
export const startAudio=serviceExports.startAudio;
export const checkAudio=serviceExports.checkAudio;
export const simulationAction=serviceExports.simulationAction;
export const PROBE_BUILD=serviceExports.PROBE_BUILD;
export const SIMULATOR_BUILD=serviceExports.SIMULATOR_BUILD;
