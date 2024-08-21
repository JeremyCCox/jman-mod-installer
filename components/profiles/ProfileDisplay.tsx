import {useSearchParams} from "react-router-dom";
import {useEffect, useState} from "react";
import LocalProfile from "../LocalProfile.tsx";
import RemoteProfileInfo from "../RemoteProfileInfo.tsx";

export default function ProfileDisplay(){
    const [searchParams] = useSearchParams();
    const [profileInfo, setProfileInfo] = useState({profile:searchParams.get("profile"),source:searchParams.get("source")})
    useEffect(()=>{
        setProfileInfo({profile:searchParams.get("profile"),source:searchParams.get("source")})
    },[searchParams])
    return(
        <>
            {!profileInfo.profile?
                <div className={'w-full border-black border'}>
                    <h3>Welcome to the JMAN Mod loader!</h3>
                </div>
                :
                <div className={'w-full flex flex-col'}>
                    {profileInfo.source === "local"?
                        <LocalProfile profileName={profileInfo.profile}/>
                        :
                        <RemoteProfileInfo profileName={profileInfo.profile}/>
                    }
                </div>
            }
        </>
    )
}