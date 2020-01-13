'use strict'

const libp2p = require('libp2p');
const TCP = require('libp2p-tcp');
const Mplex = require('libp2p-mplex');
const PeerInfo = require('peer-info');
const defaultsDeep = require('@nodeutils/defaults-deep');
const waterfall = require('async/waterfall');
const once = require('once');
const pull = require('pull-stream')
const Ping = require('libp2p-ping');

var RUST_PEER = "/ip4/127.0.0.1/tcp/30000/p2p/QmTESkr2vWDCKqiHVsyvf4iRQCBgvNDqBJ6P3yTTDb6haw";

if (process.argv.length > 2) {
    RUST_PEER = process.argv[2];
    console.log("Using peer from argument: ", RUST_PEER);
}

function enablePing(node, peer) {
    let p = new Ping(node, peer);
    p.start();
}

class MyBundle extends libp2p {
  constructor(_options) {
      const defaults = {
          modules: {
              transport: [TCP], // TODO: try udp? try websocket?
              streamMuxer: [Mplex],
              connEncryption: []
          }
      };

    super(defaultsDeep(_options, defaults))
  }
}

function createNode(callback) {
    let node;

    waterfall([
        (cb) => {
            cb = once(cb);
            PeerInfo.create().then((pi) => cb(null, pi)).catch((err) => cb(err))
        },
        (peerInfo, cb) => {
            console.log("Local peer created " + peerInfo.id.toB58String());
            peerInfo.multiaddrs.add('/ip4/127.0.0.1/tcp/0');
            node = new MyBundle({
                peerInfo
            });
            Ping.mount(node);
            node.on('peer:discovery', (peer) => {
                console.log('Discovered peer:', peer.id.toB58String());
            });
            node.on('peer:connect', (peer) => {
                console.log('Connection established to:', peer.id.toB58String());
                // enablePing(node, peer);
            });
            node.on('connection:start', (peerInfo) => {
                console.log('Connection started to:', peerInfo.id.toB58String())
            });
            node.on('connection:end', (peerInfo) => {
                console.log('Connection ended with:', peerInfo.id.toB58String())
            });
            node.on('error', (err) => {
                console.error('Node received error:', err);
            });
            node.start(cb);
        },
        (cb) => {
            console.log("node started");
            console.log("will dial " + RUST_PEER);

            node.dial(RUST_PEER, cb)
        },
        (cb) => {
            console.log("node dialed");
        },
        (cb) => {
            process.stdin.setEncoding('utf8');
            process.openStdin().on('data', (chunk) => {
                let data = chunk.toString();
                console.log("will send to node", data);
                let protocol = "/plain-protocol/1.0.0"

                node.dialProtocol(RUST_PEER, protocol,(err, conn) => {
                    console.log(err)
                    if (err) { throw err }
                    pull(pull.values(['from 1 to 2']), conn)
                    cb
                })
            });
            cb()
        }
    ], (err) => callback(err, node))
}

createNode((err) => {
  if (err) {
    console.log('\nError:', JSON.stringify(err));
    throw err
  }
});
