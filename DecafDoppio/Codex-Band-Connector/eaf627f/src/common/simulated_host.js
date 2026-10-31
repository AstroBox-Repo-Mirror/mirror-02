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

export function createSimulatedHost (e = () => Date.now()) {
  let t = "simulator-only-" + "f".repeat(48),
    r = !0,
    n = !0,
    a = 0,
    s = [];
  function o(t, r, i) {
    let n = "sim-" + ++a,
      o = Array.from(i),
      d = [];
    for (let e = 0; e < o.length; e += 700) d.push(o.slice(e, e + 700).join(""));
    s.push({
      id: n,
      kind: t,
      title: r,
      pages: d.length ? d : [""],
      reviewed: new Set(),
      state: "pending",
      expiresAt: e() + 12e4
    });
  }
  function d() {
    return s.filter(t => "pending" === t.state && e() < t.expiresAt).map(t => ({
      id: t.id,
      kind: t.kind,
      title: t.title,
      pages: t.pages.length,
      session: "simulation",
      expiresAt: t.expiresAt,
      project: "模拟项目",
      createdAt: e()
    }));
  }
  return o("approval", "请求批准", "是否允许读取项目文件，以检查当前设置？"), o("approval", "请求批准", "这是一段用于验证完整阅读与分页的请求说明。".repeat(110)), o("question", "Codex 有个问题", "要保留现有设置吗？"), {
    receive(a) {
      let o = JSON.parse(a);
      if (!n) return {
        v: 1,
        type: "unavailable"
      };
      if ("hello" === o.type) return {
        v: 1,
        type: "hello",
        ticket: t,
        hostId: "aiot-simulator",
        enabled: r,
        requests: d(),
        serverTime: e(),
        experimentalAudioValidated: !1
      };
      if (o.ticket !== t) return {
        v: 1,
        type: "error",
        error: "invalid_ticket"
      };
      let l = {
        v: 1,
        ticket: t,
        serverTime: e()
      };
      if ("resume" === o.type && o.manual) return r = !0, i(i({}, l), {}, {
        type: "resumed",
        requests: d()
      });
      if ("stop" === o.type) return r = !1, i(i({}, l), {}, {
        type: "stopped"
      });
      if (!r) return {
        v: 1,
        type: "unavailable"
      };
      if ("ping" === o.type) return i(i({}, l), {}, {
        type: "pong",
        nonce: o.nonce
      });
      if ("list" === o.type) return i(i({}, l), {}, {
        type: "list",
        requests: d()
      });
      let u = s.find(e => e.id === o.id);
      return u ? (e() >= u.expiresAt && (u.state = "expired"), "pending" !== u.state) ? i(i({}, l), {}, {
        type: "result",
        id: u.id,
        state: u.state
      }) : "detail" === o.type && Number.isInteger(o.page) && o.page >= 0 && o.page < u.pages.length ? (u.reviewed.add(o.page), i(i({}, l), {}, {
        type: "detail",
        id: u.id,
        kind: u.kind,
        title: u.title,
        session: "simulation",
        page: o.page,
        pages: u.pages.length,
        content: u.pages[o.page],
        presentation: "human",
        expiresAt: u.expiresAt,
        canApprove: u.reviewed.size === u.pages.length
      })) : "decision" === o.type && "approval" === u.kind && ["allow", "deny"].includes(o.decision) ? "allow" === o.decision && u.reviewed.size !== u.pages.length ? i(i({}, l), {}, {
        type: "error",
        error: "review_required",
        id: u.id
      }) : (u.state = o.decision, i(i({}, l), {}, {
        type: "result",
        id: u.id,
        state: u.state
      })) : i(i({}, l), {}, {
        type: "error",
        error: "invalid_command",
        id: u.id
      }) : i(i({}, l), {}, {
        type: "error",
        error: "not_found",
        id: o.id
      });
    },
    newRequest() {
      o("approval", "请求批准", "是否允许进行一次无害测试？");
    },
    expire() {
      s.forEach(t => {
        t.expiresAt = e() - 1;
      });
    },
    disconnect() {
      n = !1;
    },
    reconnect() {
      n = !0;
    }
  };
}
