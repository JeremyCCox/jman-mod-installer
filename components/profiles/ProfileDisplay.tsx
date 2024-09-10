import {useSearchParams} from "react-router-dom";
import {useEffect, useState} from "react";
import LocalProfile from "../LocalProfile.tsx";
import RemoteProfileInfo from "../RemoteProfileInfo.tsx";
import Addons from "../Addons.tsx";

export default function ProfileDisplay(){
    const [searchParams] = useSearchParams();
    const [profileInfo, setProfileInfo] = useState({profile:searchParams.get("profile"),source:searchParams.get("source"),page:searchParams.get("page")})
    useEffect(()=>{
        setProfileInfo({profile:searchParams.get("profile"),source:searchParams.get("source"),page:searchParams.get("page")})
    },[searchParams])
    switch(profileInfo.page){
        case("resourcepacks"):
            return(
              <Addons addonType={"ResourcePack"}/>
            )
        case("mods"):
            return(
                <Addons addonType={"Mod"}/>
            )
        case("profile"):
            if(profileInfo.profile){
                return (
                    <div className={'w-full h-full'}>
                        {profileInfo.source === "local"?
                            <LocalProfile profileName={profileInfo.profile}/>
                            :
                            <RemoteProfileInfo profileName={profileInfo.profile}/>
                        }
                    </div>
                )
            }else{
                return(
                    <div className={'w-full border-black border'}>
                        <h3>Please select a profile</h3>
                    </div>
                )
            }
        default:
            return (
                <div className={'w-full border-black border'}>
                    <h3>Welcome to the JMAN Mod loader!</h3>
                </div>
            )

    }
}