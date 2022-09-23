import trio
from trio_websocket import WebSocketRequest, WebSocketConnection, open_websocket_url

import logging; logging.basicConfig(level=logging.DEBUG)

class PEP8violation:
    s: WebSocketConnection

w = PEP8violation()

async def input_msgs():
    while True:
        msg = await trio.to_thread.run_sync(input)
        await w.s.send_message(msg)
        await trio.sleep(1)

async def open_websocket():
    async with open_websocket_url("ws://localhost:6969") as w.s:
        while True:
            print(await w.s.get_message())

async def main():
    async with trio.open_nursery() as nursery:
        nursery.start_soon(open_websocket)
        nursery.start_soon(input_msgs)
    print('wow')

trio.run(main)