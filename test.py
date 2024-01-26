import asyncio
from etcd_client import Client

etcd_client = Client(["http://localhost:2379"])


async def main():
    async with etcd_client.connect() as etcd:
        await etcd.put("wow2/abc", "abc")

if __name__ == "__main__":
    asyncio.run(main())
