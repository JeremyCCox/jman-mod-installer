import GameDir from "./GameDir";
import {InstallerProfile} from "@my-types/*";
import {useState} from "react";

export default function UserLogin({handleSubmit,config}:{handleSubmit:any,config?:InstallerProfile}){
    const [username, setUsername] = useState(config?.sftpUsername||"")
    const [server, setServer] = useState(config?.sftpServer||"")
    const [port, setPort] = useState(config?.sftpPort||"")

    return(
        <div className={'text-center m-auto w-80'}>
            <h1 className={'text-4xl font-bold font-mono mt-[15vh] mb-[5vh]'}>JMAN MOD MANAGER</h1>
            <form onSubmit={handleSubmit} className={'grid'}>
                <input type={'text'} name={'sftpUsername'} autoComplete={"username"} value={username} className={"py-3 my-2 "} placeholder={"username"} onChange={(e)=>{setUsername(e.currentTarget.value)}}/>
                <input type={'password'} name={'sftpPassword'} autoComplete={"current-password"} className={"py-3 my-2 "} placeholder={"password"} />
                <div className={"flex w-full"}>
                    <input type={'text'} name={"sftpServer"} className={"py-3 my-2 grow "} placeholder={"Profile Server"} value={server} onChange={(e)=>{setServer(e.currentTarget.value)}}/>
                    <input type={'text'} name={"sftpPort"} className={"py-3 my-2 w-24 "} placeholder={"Port"} value={port} onChange={(e)=>{setPort(e.currentTarget.value)}}/>
                </div>
                <GameDir/>
                <button>
                    Sign In
                </button>
            </form>
        </div>
    )
}