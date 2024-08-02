import React from "react";
import {useQueryClient} from "react-query";
import {RemoteProfileType} from "./RemoteInfo.tsx";
import CompareLocalProfile from "./CompareLocalProfile.tsx";
import {invoke} from "@tauri-apps/api";

export default function RemoteProfile({profile,setMessage,setLoading, path}:Readonly<{profile:RemoteProfileType,setMessage:any,setLoading:any,path:string}>){

    const queryClient = useQueryClient();

    const installProfile= async (e:React.MouseEvent<HTMLButtonElement, MouseEvent>) => {
        setLoading(true)
        console.log(e.currentTarget.name);
        // @ts-ignore
        invoke('download_sftp_profile', {basePath: path, profileName: e.currentTarget.name}).then((res:string)=>{
            setMessage(res)
            queryClient.refetchQueries("profiles")
        }).catch((err:string)=>{
            setMessage(err)
        }).finally(()=>{
            setLoading(false)
        })
    }
    return(

            <div key={profile.name} className={"w-1/3 h-50 px-4 flex flex-col"}>
                <h4 className={'text-xl font-bold text-center'}>{profile.name}</h4>
                <button  name={profile.name} onClick={installProfile}>
                    Download Profile
                </button>
                <div className={'h-60 w-full border-2 border-black overflow-y-scroll'}>
                    {profile.mods?.map(mod=>{
                        return(
                            <p className={''} key={mod}>
                                {mod}
                            </p>
                        )
                    })}
                </div>
                <CompareLocalProfile profile={profile} key={profile.name} path={path}/>
            </div>


    )
}