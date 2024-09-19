import {ProfileAddon} from "../../lib/types";
import {useSearchParams} from "react-router-dom";

export default function AddonRow({addon,isNew=false}:Readonly<{ addon:ProfileAddon,isNew?:boolean}>){
    const [searchParams,setSearchParams] = useSearchParams()
    const handleClick =()=>{
        searchParams.set("addon_name",addon.name)
        searchParams.set("isNew",isNew?"t":"f")
        setSearchParams(searchParams)
    }
    return(
        <button onClick={handleClick} style={{backgroundColor:searchParams.get("addon_name")===addon.name?"lime":undefined}} type={'button'} className={'my-1 bg-red-500 p-2 hover:bg-red-400 active:bg-red-300 hover:cursor-pointer text-left'}>
            {addon.name}
        </button>
    )
}