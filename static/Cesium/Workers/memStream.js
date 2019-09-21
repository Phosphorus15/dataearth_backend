/**
 * Cesium - https://github.com/AnalyticalGraphicsInc/cesium
 *
 * Copyright 2011-2017 Cesium Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 * Columbus View (Pat. Pend.)
 *
 * Portions licensed separately.
 * See https://github.com/AnalyticalGraphicsInc/cesium/blob/master/LICENSE.md for full licensing details.
 */
!function(){define("Workers/memStream",[],function(){"use strict";function t(t){this.curOffset=0,this.data=t,this.bLittleEndian=this.isLittleEndian()}return t.prototype.TWO_POW_MINUS23=Math.pow(2,-23),t.prototype.TWO_POW_MINUS126=Math.pow(2,-126),t.prototype.isLittleEndian=function(){var t=new ArrayBuffer(2),r=new Uint8Array(t),e=new Uint16Array(t);return r[0]=1,1===e[0]},t.prototype.readByte_1=function(){return 255&this.data[this.curOffset++]},t.prototype.readUChar8_1=function(){return 255&this.data[this.curOffset++]},t.prototype.readUInt32_1=function(){var t=this.readByte();return t|=this.readByte()<<8,(t|=this.readByte()<<16)|this.readByte()<<24},t.prototype.readInt32_1=function(){var t=this.readByte();return t|=this.readByte()<<8,(t|=this.readByte()<<16)|this.readByte()<<24},t.prototype.readFloat_1=function(){var t=this.readByte();t+=this.readByte()<<8;var r=this.readByte(),e=this.readByte();t+=(127&r)<<16;var a=(127&e)<<1|(128&r)>>>7,i=128&e?-1:1;return 255===a?0!==t?NaN:i*(1/0):a>0?i*(1+t*this.TWO_POW_MINUS23)*Math.pow(2,a-127):0!==t?i*t*this.TWO_POW_MINUS126:0*i},t.prototype.readUChar8=function(){var t=new Uint8Array(this.data,this.curOffset,1);return this.curOffset+=1,t[0]},t.prototype.readUInt32=function(){var t=new DataView(this.data,this.curOffset,4),r=t.getUint32(0,!0);return this.curOffset+=4,r},t.prototype.readInt32=function(){var t=new DataView(this.data,this.curOffset,4);return this.curOffset+=4,t.getInt32(0,!0)},t.prototype.readInt16=function(){var t=new DataView(this.data,this.curOffset,2);return this.curOffset+=2,t.getInt16(0,!0)},t.prototype.readUInt16=function(){var t=new DataView(this.data,this.curOffset,2);return this.curOffset+=2,t.getUint16(0,!0)},t.prototype.readFloat=function(){var t=new DataView(this.data,this.curOffset,4);return this.curOffset+=4,t.getFloat32(0,!0)},t.prototype.readDouble=function(){var t=new DataView(this.data,this.curOffset,8);return this.curOffset+=8,t.getFloat64(0,!0)},t.prototype.readFloat32Array=function(t){var r=4*t,e=new DataView(this.data,this.curOffset,r);return this.curOffset+=r,e},t.prototype.readFloat64Array=function(t){var r=8*t,e=new DataView(this.data,this.curOffset,r);return this.curOffset+=r,e},t.prototype.readUInt16Array=function(t){var r=2*t,e=new DataView(this.data,this.curOffset,r);return this.curOffset+=r,e},t.prototype.readUInt32Array=function(t){var r=4*t,e=new DataView(this.data,this.curOffset,r);return this.curOffset+=r,e},t.prototype.readUChar8Array=function(t){var r=1*t,e=new DataView(this.data,this.curOffset,r);return this.curOffset+=r,e},t.prototype.readUChar8Array2=function(t){var r=new Uint8Array(this.data,this.curOffset,t);return this.curOffset+=t,r},t.prototype.readString=function(){for(var t=this.readUInt32(),r=new Uint8Array(this.data,this.curOffset,t),e="",a=0;a<t;a++)e+=String.fromCharCode(r[a]);return this.curOffset+=t,e},t})}();